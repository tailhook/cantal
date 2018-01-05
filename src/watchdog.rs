use std::process::exit;

/// This is a guard with exits with specified code on Drop
#[must_use="This guard must be put into a variable it will exit immediately"]
pub struct ExitOnReturn(pub i32);


impl Drop for ExitOnReturn {
    fn drop(&mut self) {
        exit(self.0);
    }
}
