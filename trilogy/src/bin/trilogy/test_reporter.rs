use colored::Colorize;
use trilogy::{Location, TestDescription, TestReporter};
use trilogy_vm::{ErrorKind, Value};

/// Test reporter that prints test results to standard output.
#[derive(Default)]
pub struct Stdout {
    depth: usize,
    passes: usize,
    fails: usize,
}

impl Stdout {
    fn indent(&self) -> String {
        "  ".repeat(self.depth)
    }

    pub fn is_error(&self) -> bool {
        self.fails != 0
    }
}

impl TestReporter for Stdout {
    fn enter_document(&mut self, location: &Location) {
        println!("{}", location.to_string().magenta());
        self.depth += 1;
    }

    fn enter_module(&mut self, submodule: &str) {
        println!("{}module {submodule}", self.indent());
        self.depth += 1;
    }

    fn test_result(
        &mut self,
        test_name: &str,
        TestDescription { negated }: TestDescription,
        result: Result<trilogy_vm::Value, trilogy_vm::Error>,
    ) {
        let result = match result {
            Ok(value) if !negated => Ok(Ok(value)),
            Ok(value) => Ok(Err(value)),
            Err(error) => match error.kind {
                ErrorKind::RuntimeError(value) if negated => Ok(Ok(value)),
                ErrorKind::RuntimeError(value) => Ok(Err(value)),
                _ => Err(error),
            },
        };

        let (icon, result_summary) = match result {
            Ok(Ok(value)) => {
                self.passes += 1;
                let summary = match value {
                    Value::Unit => format!("{}", "passed".green()),
                    value => format!(
                        "{} {}",
                        "passed".green(),
                        format!("(result: {value})").bright_white()
                    ),
                };
                let icon = format!("{}", if negated { "✗" } else { "✓" }.green());
                (icon, summary)
            }
            Ok(Err(value)) => {
                self.fails += 1;
                let summary = match value {
                    Value::Unit => format!("{}", "failed".red()),
                    value => format!(
                        "{} {}",
                        "failed".red(),
                        format!("(result: {value})").bright_white()
                    ),
                };
                let icon = format!("{}", if negated { "✓" } else { "✗" }.red());
                (icon, summary)
            }
            Err(error) => {
                self.fails += 1;
                let summary = format!("{} ({error})", "crashed".black().on_red());
                let icon = format!("{}", "✗".red());
                (icon, summary)
            }
        };

        println!(
            "{}{icon} {}{}: {result_summary}",
            self.indent(),
            if negated { "not " } else { "" },
            test_name.blue(),
        );
    }

    fn exit_module(&mut self) {
        self.depth -= 1;
    }

    fn exit_document(&mut self) {
        self.depth -= 1;
    }

    fn finish(&mut self) {
        println!();
        println!("Total tests run: {}", self.fails + self.passes);
        println!("Tests passed: {}", self.passes.to_string().green());
        println!(
            "Tests failed: {}",
            if self.fails == 0 {
                "0".green()
            } else {
                self.fails.to_string().red()
            }
        );
    }
}
