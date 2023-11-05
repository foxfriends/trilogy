use crate::Location;
use trilogy_vm::Value;

/// A type that can receive the results of running a Trilogy program's
/// test suite.
///
/// Trilogy tests are run sequentially, one file at a time.
#[allow(unused_variables)]
pub trait TestReporter {
    /// Called at the beginning of the test run.
    fn begin(&mut self) {}

    /// Called each time a new file's tests are started.
    fn enter_document(&mut self, location: &Location) {}

    /// Called each time a module within a file is entered.
    fn enter_module(&mut self, name: &str) {}

    /// Called with the result of each test that is run.
    fn test_result(&mut self, test_name: &str, result: Result<Value, trilogy_vm::Error>) {}

    /// Called at the end of running all of a module's tests.
    fn exit_module(&mut self) {}

    /// Called at the end of running all of a file's tests.
    fn exit_document(&mut self) {}

    /// Called at the end of the test run.
    fn finish(&mut self) {}
}
