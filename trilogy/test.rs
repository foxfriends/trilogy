use colored::Colorize;
use serde::Deserialize;
use std::collections::HashSet;
use std::env::var;
use std::fs::{File, exists, read_dir, read_to_string};
use std::io::{self, Read, Write, stdout};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{Sender, channel};
use std::time::{Duration, Instant};
use threadpool::ThreadPool;

struct Report {
    path: PathBuf,
    trilogy_exit_code: i32,
    trilogy_stderr: String,
    trilogy_compile_time: Duration,
    clang_output: Option<Output>,
    clang_compile_time: Duration,
    program_output: Option<Output>,
    program_time: Duration,
    expected: Expectation,
}

const fn const_true() -> bool {
    true
}

#[derive(Deserialize)]
struct Expectation {
    #[serde(default)]
    test: bool,
    #[serde(default)]
    filter_prefix: Vec<String>,
    #[serde(default)]
    exit: i32,
    #[serde(default)]
    output: String,
    #[serde(default)]
    stderr: bool,
    #[serde(default = "const_true")]
    compile: bool,
}

impl Default for Expectation {
    fn default() -> Self {
        Self {
            test: false,
            filter_prefix: vec![],
            exit: 0,
            output: String::new(),
            stderr: false,
            compile: true,
        }
    }
}

impl Report {
    fn is_success(&self) -> bool {
        if !self.expected.compile {
            return self.trilogy_exit_code != 0;
        }

        self.trilogy_exit_code == 0
            && self
                .clang_output
                .as_ref()
                .map(|out| out.status.success())
                .unwrap_or(false)
            && self
                .program_output
                .as_ref()
                .map(|output| {
                    output
                        .status
                        .code()
                        .map(|code| code == self.expected.exit)
                        .unwrap_or(false)
                        && (self.expected.test
                            || (output.stdout == self.expected.output.as_bytes()
                                && if self.expected.stderr {
                                    !output.stderr.is_empty()
                                } else {
                                    output.stderr.is_empty()
                                }))
                })
                .unwrap_or(false)
    }

    fn print_summary(&self) -> io::Result<()> {
        let mut stdout = stdout().lock();
        writeln!(
            stdout,
            "test {} ... {} ({:?} trilogy; {:?} clang; {:?} native)",
            self.path.file_name().unwrap().to_string_lossy(),
            if self.is_success() {
                "ok".green()
            } else {
                "FAILED".red()
            },
            self.trilogy_compile_time,
            self.clang_compile_time,
            self.program_time,
        )?;
        Ok(())
    }

    fn print_failure(&self) -> io::Result<()> {
        let mut stdout = stdout().lock();
        if !self.expected.compile {
            writeln!(
                stdout,
                "{} was expected not to compile",
                self.path.file_name().unwrap().to_string_lossy(),
            )?;
            return Ok(());
        }
        if self.trilogy_exit_code != 0 {
            writeln!(
                stdout,
                "{} failed to compile\n---- trilogy error output ----\n{}",
                self.path.file_name().unwrap().to_string_lossy(),
                self.trilogy_stderr,
            )?;
            return Ok(());
        }

        let clang_output = self.clang_output.as_ref().unwrap();
        if !clang_output.status.success() {
            writeln!(
                stdout,
                "{} generated invalid llvm ir",
                self.path.file_name().unwrap().to_string_lossy()
            )?;
            if !clang_output.stdout.is_empty() {
                writeln!(
                    stdout,
                    "---- clang output ----\n{}",
                    std::str::from_utf8(&clang_output.stdout).unwrap_or("non UTF-8 output"),
                )?;
            }
            if !clang_output.stderr.is_empty() {
                writeln!(
                    stdout,
                    "---- clang error output ----\n{}",
                    std::str::from_utf8(&clang_output.stderr).unwrap_or("non UTF-8 output"),
                )?;
            }
            return Ok(());
        }

        let program_output = self.program_output.as_ref().unwrap();

        if program_output
            .status
            .code()
            .map(|code| code != self.expected.exit)
            .unwrap_or(true)
        {
            writeln!(
                stdout,
                "{} {} (expected {})",
                self.path.file_name().unwrap().to_string_lossy(),
                program_output
                    .status
                    .code()
                    .map(|code| { format!("exited {code}") })
                    .unwrap_or_else(|| {
                        #[cfg(unix)]
                        {
                            let signal = if let Some(sig) = program_output.status.signal() {
                                format!("terminated by signal {sig}")
                            } else if let Some(sig) = program_output.status.stopped_signal() {
                                format!("stopped by signal {sig}")
                            } else {
                                "terminated unexpectedly".to_owned()
                            };
                            if program_output.status.core_dumped() {
                                format!("{signal} (core dumped)")
                            } else {
                                signal
                            }
                        }

                        #[cfg(not(unix))]
                        {
                            "unknown exit code".to_owned()
                        }
                    }),
                self.expected.exit,
            )?;
        } else {
            writeln!(
                stdout,
                "{} output differs from expectation",
                self.path.file_name().unwrap().to_string_lossy()
            )?;
        }
        let output = std::str::from_utf8(&program_output.stdout).unwrap_or("non UTF-8 output");

        if self.expected.test {
            writeln!(stdout, "---- test report ----\n{output}")?;
            return Ok(());
        }

        if output != self.expected.output {
            writeln!(
                stdout,
                "---- expected output ----\n{}",
                self.expected.output,
            )?;
            writeln!(stdout, "---- actual output ----\n{output}")?;
        }
        if self.expected.stderr {
            if program_output.stderr.is_empty() {
                writeln!(stdout, "expected error output")?;
            }
        } else if !program_output.stderr.is_empty() {
            writeln!(
                stdout,
                "---- unexpected error output ----\n{}",
                std::str::from_utf8(&program_output.stderr).unwrap_or("non UTF-8 output"),
            )?;
        }

        Ok(())
    }
}

fn test_case(path: PathBuf, done: Sender<Report>) {
    let opt = var("TRITEST_OPT").unwrap_or_else(|_| "-O0".to_owned());
    let trilogy = env!("CARGO_BIN_EXE_trilogy");
    let memcheck = var("TRITEST_MEMCHECK")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(false);

    let mut expected = Expectation::default();
    if exists(path.join("spec.toml")).unwrap() {
        expected = toml::from_str(&read_to_string(path.join("spec.toml")).unwrap()).unwrap();
    }
    let mut report = Report {
        path: path.clone(),
        trilogy_exit_code: 0,
        trilogy_stderr: String::new(),
        trilogy_compile_time: Duration::ZERO,
        clang_output: None,
        clang_compile_time: Duration::ZERO,
        program_output: None,
        program_time: Duration::ZERO,
        expected,
    };

    'test: {
        let tri = path.join("main.tri");
        let ll = path.join("main.ll");
        let program = path.join("a.out");
        let ll_file = File::create(&ll).unwrap();
        let mut trilogy_command = Command::new(trilogy);
        trilogy_command
            .args(["compile", tri.to_str().unwrap()])
            .stdout(ll_file)
            .stderr(Stdio::piped());
        if report.expected.test {
            trilogy_command.arg("--test");
            for prefix in &report.expected.filter_prefix {
                trilogy_command.args(["--prefix", prefix]);
            }
        }
        let start = Instant::now();
        let mut trilogy_compile = trilogy_command.spawn().unwrap();
        let mut stderr = trilogy_compile.stderr.take().unwrap();
        report.trilogy_exit_code = trilogy_compile.wait().unwrap().code().unwrap();
        report.trilogy_compile_time = start.elapsed();
        stderr.read_to_string(&mut report.trilogy_stderr).unwrap();
        if report.trilogy_exit_code != 0 {
            break 'test;
        }

        let clang = var("LLVM_SYS_191_PREFIX")
            .ok()
            .map(|pref| pref + "/bin/")
            .unwrap_or_else(|| "".to_owned())
            + "clang";
        let mut clang_command = Command::new(clang);
        clang_command.args([
            ll.to_str().unwrap(),
            "-g",
            opt.as_str(),
            "-rdynamic",
            "-o",
            program.to_str().unwrap(),
        ]);
        let start = Instant::now();
        report.clang_output = Some(clang_command.output().unwrap());
        report.clang_compile_time = start.elapsed();
        if !report.clang_output.as_ref().unwrap().status.success() {
            break 'test;
        }

        let mut program_command = if memcheck {
            let memcheck = path.join("memcheck");
            let mut cmd = Command::new("valgrind");
            cmd.args([
                &format!("--log-file={}", memcheck.display()),
                "--error-exitcode=97",
                "--errors-for-leak-kinds=definite,possible",
                "--show-leak-kinds=all",
                "--",
                program.to_str().unwrap(),
            ]);
            cmd
        } else {
            Command::new(program)
        };
        let start = Instant::now();
        report.program_output = Some(program_command.output().unwrap());
        report.program_time = start.elapsed();
    }

    done.send(report).unwrap();
}

fn main() {
    let quiet = var("TRITEST_QUIET")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(false);
    let thread_count = var("TRITEST_NUM_THREADS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(num_cpus::get);

    let pool = ThreadPool::new(thread_count);
    let start = Instant::now();
    let mut expected = HashSet::new();

    let rx = {
        let (tx, rx) = channel();
        for case in read_dir("../testsuite").unwrap() {
            let case = case.unwrap();
            if case.file_type().unwrap().is_dir() {
                let tx = tx.clone();
                expected.insert(case.path());
                pool.execute(move || test_case(case.path(), tx));
            }
        }
        rx
    };

    let mut failures = vec![];
    let mut successes = 0;
    while let Ok(report) = rx.recv() {
        expected.remove(&report.path);
        if !quiet {
            report.print_summary().unwrap();
        }
        if report.is_success() {
            successes += 1;
        } else {
            failures.push(report);
        }
    }

    let duration = start.elapsed();
    if !failures.is_empty() || !expected.is_empty() {
        println!("\nfailures:\n");
        for report in &failures {
            report.print_failure().unwrap();
        }
        for path in &expected {
            println!("{} did not succeed", path.display());
        }

        println!(
            "\ntest result: {}. {} passed; {} failed; {} crashed; finished in {:?}\n",
            "FAILED".red(),
            successes,
            failures.len(),
            expected.len(),
            duration
        );
        std::process::exit(101);
    }

    println!(
        "\ntest result: {}. {} passed; 0 failed; 0 crashed; finished in {:?}\n",
        "ok".green(),
        successes,
        duration
    );
    std::process::exit(0);
}
