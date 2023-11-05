use colored::Colorize;
use trilogy::{Location, TestReporter};
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
        println!("{location}");
        self.depth += 1;
    }

    fn enter_module(&mut self, submodule: &str) {
        println!("{}module {submodule}", self.indent());
        self.depth += 1;
    }

    fn test_result(
        &mut self,
        test_name: &str,
        result: Result<trilogy_vm::Value, trilogy_vm::Error>,
    ) {
        let (icon, result_summary) = match result {
            Ok(value) => {
                self.passes += 1;
                let summary = match value {
                    Value::Unit => format!("{}", "passed".green()),
                    value => format!("{} (result: {value})", "passed".green()),
                };
                let icon = format!("{}", "✓".green());
                (icon, summary)
            }
            Err(error) => {
                self.fails += 1;
                let summary = match error.kind {
                    ErrorKind::RuntimeError(error) => format!("{} ({error})", "failed".red()),
                    error => format!("{} ({error})", "crashed".on_red()),
                };
                let icon = format!("{}", "✗".red());
                (icon, summary)
            }
        };

        println!("{}{icon} {test_name}: {result_summary}", self.indent());
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
