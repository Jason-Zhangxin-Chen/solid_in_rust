// Error, std::error::Error.
// The base trait for error types. Provides the source() chain for error causation.
// Combined with Display and Debug. thiserror auto-implements this pattern.
fn error() {
    use std::fmt;

    #[derive(Debug)]
    struct ParseConfigError { line: usize, msg: String }

    impl fmt::Display for ParseConfigError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "parse error on line {}: {}", self.line, self.msg)
        }
    }

    impl std::error::Error for ParseConfigError {}  // source() defaults to None

    // Walk the error chain:
    fn print_chain(e: &dyn std::error::Error) {
        eprintln!("error: {e}");
        let mut src = e.source();
        while let Some(cause) = src {
            eprintln!("  caused by: {cause}");
            src = cause.source();
        }
    }
}