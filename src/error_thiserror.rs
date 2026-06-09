// Quick reference cheatsheet
// Attribute                    Effect
// #[error("msg {field}")]      implements Display
// #[from]                      implements From + wires source()
// #[source]                    wires source() chain, no From
// #[error(transparent)]        delegates Display and source() to inner
// Works on enums               ✅ most common
// Works on structs             ✅ for single-purpose errors
// The core philosophy: thiserror is for libraries — it gives callers structured, matchable
// error types. For application code (binaries), anyhow is the companion crate that trades
// matchability for ergonomic one-liner error handling.

// Here is an example of using the `thiserror` crate to define custom error types in Rust.
// # Cargo.toml
// [package]
// name = "mylib"
// version = "0.1.0"
// edition = "2021"
//
// [dependencies]
// thiserror = "1"

// Basic error enum with messages.
fn basic_error_enum() {
    use thiserror::Error;

    struct User {
        id: u64,
    }

    #[derive(Debug, Error)]
    pub enum AppError {
        // Static message
        #[error("not found")]
        NotFound,

        // Dynamic message with field interpolation
        #[error("user `{id}` not found")]
        UserNotFound { id: u64 },

        // Tuple variant — positional field uses index
        #[error("invalid input: `{0}`")]
        InvalidInput(String),

        // Multiple fields
        #[error("value {value} is out of range [{min}, {max}]")]
        OutOfRange { value: i32, min: i32, max: i32 },
    }

    fn find_user(id: u64) -> Result<User, AppError> {
        if id == 0 {
            return Err(AppError::UserNotFound { id });
        }
        Ok(User { id })
    }
}

// Wrapping external errors with #[from] and source chaining.
// #[from] implements From<ExternalError> for AppError automatically, enabling ? to work seamlessly.

fn error_chaining() {
    use thiserror::Error;
    use std::io;
    use std::num::ParseIntError;

    #[derive(Debug, Error)]
    pub enum AppError {
        #[error("io error: {0}")]
        Io(#[from] io::Error),

        #[error("parse error: {0}")]
        Parse(#[from] ParseIntError),

        #[error("invalid input: {0}")]
        InvalidInput(String),
    }

    // Now ? auto-converts io::Error and ParseIntError:
    fn read_number(path: &str) -> Result<i32, AppError> {
        let content = std::fs::read_to_string(path)?;  // io::Error -> AppError::Io
        let n = content.trim().parse::<i32>()?;         // ParseIntError -> AppError::Parse
        Ok(n)
    }
}

// Preserving the source with #[source].
// #[source] exposes the inner error via the std::error::Error::source() chain, without generating
// From. Use this when you want to add context but not auto-convert with ?.
fn preserve_error_source() {
    use thiserror::Error;

    struct Config(String);

    #[derive(Debug, Error)]
    pub enum AppError {
        // #[from] implies #[source] — both source() and From are implemented
        // #[error("database failure")]
        // Database(#[from] DbError),

        // #[source] only — no From, but source() chain is preserved
        #[error("config load failed for `{path}`")]
        ConfigLoad {
            path: String,
            #[source]
            cause: std::io::Error,
        },
    }

    fn load_config(path: &str) -> Result<Config, AppError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::ConfigLoad {
                path: path.to_owned(),
                cause: e,             // manually wrap — no ? shortcut here
            })?;

        Ok(Config(content))
    }
}

// layered / module-specific errors.
// Real libraries have multiple subsystems. Give each its own error type, then compose them into
// a top-level error.
fn modulized_error() {
    use thiserror::Error;
    // src/db/error.rs
    #[derive(Debug, Error)]
    pub enum DbError {
        #[error("connection refused at `{addr}`")]
        ConnectionRefused { addr: String },

        #[error("query failed: {0}")]
        QueryFailed(String),

        #[error("record not found: id={0}")]
        NotFound(u64),
    }

    // src/auth/error.rs
    #[derive(Debug, Error)]
    pub enum AuthError {
        #[error("invalid credentials")]
        InvalidCredentials,

        #[error("token expired")]
        TokenExpired,

        #[error("permission denied for resource `{resource}`")]
        PermissionDenied { resource: String },
    }

    // src/errors.rs — top-level error composes subsystem errors
    #[derive(Debug, Error)]
    pub enum AppError {
        #[error("database error: {0}")]
        Db(#[from] DbError),

        #[error("auth error: {0}")]
        Auth(#[from] AuthError),

        #[error("io error: {0}")]
        Io(#[from] std::io::Error),
    }

    /*
    // Now ? works across the whole call stack:
    fn get_user_profile(token: &str, user_id: u64) -> Result<Profile, AppError> {
        verify_token(token)?;          // AuthError -> AppError::Auth
        let user = db_fetch(user_id)?; // DbError   -> AppError::Db
        Ok(build_profile(user))
    }
    */
}

// Transparent delegation with #[error(transparent)]
// When a variant wraps exactly one error and you want its Display and source() to pass through
// directly — no extra wrapping message:
fn transparent_error() {
    use std::io;
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum AppError {
        // Useful when you want the inner error's message to be the public-facing one, without
        // adding your own prefix.
        // Display and source() delegate straight to the inner io::Error
        #[error(transparent)]
        Io(#[from] io::Error),

        #[error("app logic failed: {0}")]
        Logic(String),
    }

    // io::Error's own message appears directly, no "io error: " prefix
}

// Custom Display via a method
// For complex messages that need logic, you can call a method on self:
fn custom_display() {
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("{}", self.message())]
    pub struct ValidationError {
        pub fields: Vec<String>,
    }

    impl ValidationError {
        fn message(&self) -> String {
            format!("validation failed on fields: {}", self.fields.join(", "))
        }
    }
}

// Struct errors (not just enums)
// thiserror works on structs too — useful for a single-purpose error type:
fn struct_error() {
    use thiserror::Error;
    #[derive(Debug, Error)]
    #[error("rate limit exceeded: {limit} req/s, retry after {retry_after_secs}s")]
    pub struct RateLimitError {
        pub limit: u32,
        pub retry_after_secs: u64,
    }

    fn check_rate(count: u32) -> Result<(), RateLimitError> {
        if count > 100 {
            return Err(RateLimitError { limit: 100, retry_after_secs: 60 });
        }
        Ok(())
    }
}

// Walking the error source chain.
fn walking_error_chain() {
    fn print_error_chain(e: &dyn std::error::Error) {
        eprintln!("error: {e}");
        let mut src = e.source();
        while let Some(cause) = src {
            eprintln!("  caused by: {cause}");
            src = cause.source();
        }
    }

    // Output for AppError::Db(DbError::QueryFailed(...)):
    // error: database error: query failed: syntax error near SELECT
    //   caused by: query failed: syntax error near SELECT
}