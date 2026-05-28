// ============================================================================
// ERROR HANDLING WITH ECOSYSTEM CRATES
//
// [dependencies]
// thiserror = "2"
// anyhow    = "1"
//
// thiserror — derive macros for library error types
// anyhow    — ergonomic error handling for applications
//
// Rule of thumb:
//   Library crate → thiserror  (callers need to match variants)
//   Binary / app  → anyhow     (you just want to propagate and display)
// ============================================================================

// ============================================================================
// SECTION 1 — thiserror
//
// thiserror generates the boilerplate you wrote by hand in the raw file:
//   - impl Display  (from the #[error("...")] attribute)
//   - impl Error    (source() chain from #[source] / #[from])
//
// Nothing changes at runtime — it compiles to the same code.
// The crate is a zero-overhead proc-macro; it disappears after compilation.
// ============================================================================

use thiserror::Error;
use anyhow::{anyhow, bail, ensure, Context, Result};
use std::num::ParseIntError;
use std::io;
use std::fs;

// --- 1a: A leaf error (no cause) -------------------------------------------

#[derive(Debug, Error)]
#[error("resource not found: '{resource}'")]
pub struct NotFoundError {
    pub resource: String,
}

// --- 1b: An error with a wrapped source ------------------------------------

// #[from] does two things:
//   1. impl From<io::Error> for ConfigError  (enables ? operator)
//   2. fn source() returns Some(&self.source)
//
// The field does NOT need to be named `source` — the attribute is what matters.
#[derive(Debug, Error)]
#[error("failed to load config from '{path}'")]
pub struct ConfigError {
    pub path:   String,
    #[source]   // exposes via source() but does NOT impl From
    pub source: io::Error,
}

// --- 1c: An enum covering multiple error kinds -----------------------------

#[derive(Debug, Error)]
pub enum AppError {
    // #[from] generates From<io::Error> and exposes .source()
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    // #[from] generates From<ParseIntError>
    #[error("parse error: {0}")]
    Parse(#[from] ParseIntError),

    // Domain errors with named fields — Display via the format string
    #[error("not found: {0}")]
    NotFound(String),

    #[error("user '{user}' cannot '{action}'")]
    Unauthorized { user: String, action: String },

    // Transparent: delegates both Display and source() entirely to the inner error.
    // Use when you're just forwarding another error with no added message.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// --- 1d: Nested / context errors -------------------------------------------

// A separate, focused error for one subsystem.
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("connection refused on '{host}:{port}'")]
    ConnectionRefused { host: String, port: u16 },

    #[error("query failed: {message}")]
    QueryFailed { message: String },

    #[error("migration error")]
    Migration(#[source] Box<dyn std::error::Error + Send + Sync>),
}

// A top-level error that wraps DatabaseError, forming a two-level source chain.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("database layer failed")]
    Database(#[from] DatabaseError),

    #[error("config load failed")]
    Config(#[from] ConfigError),

    #[error("upstream API error: {status}")]
    Upstream { status: u16 },
}

fn get_user(id: u32) -> Result<String, ServiceError> {
    if id == 0 {
        return Err(DatabaseError::ConnectionRefused {
            host: "db.example.com".to_string(),
            port: 5432,
        }
            .into()); // DatabaseError → ServiceError via #[from]
    }
    Ok(format!("user_{id}"))
}

// ============================================================================
// SECTION 2 — thiserror: #[error] format string features
//
// The format string in #[error("...")] is a std::fmt format string.
// It has access to all fields by name (for structs) or by position (for
// tuple variants). It can also call methods.
// ============================================================================

#[derive(Debug, Error)]
pub enum FormatDemo {
    // Named fields
    #[error("range [{min}, {max}) violated by value {value}")]
    OutOfRange { min: i32, max: i32, value: i32 },

    // Tuple field — accessed as {0}, {1}
    #[error("expected {0} items, got {1}")]
    CountMismatch(usize, usize),

    // Method call inside the format string
    // #[error("invalid hex: '{0}' (len={1})", _0, _0.len())]
    // InvalidHex(String),

    // Nested display: access a field and call .display() or similar
    #[error("wrapped: {inner}")]
    Wrapped { inner: NotFoundError },
}

// ============================================================================
// SECTION 3 — anyhow
//
// anyhow::Error is a single type that wraps ANY error implementing
// std::error::Error + Send + Sync + 'static. It also stores a backtrace
// (when RUST_BACKTRACE=1 or RUST_LIB_BACKTRACE=1).
//
// anyhow::Result<T> = Result<T, anyhow::Error>
//
// Use it inside application / binary code. Never expose it in library APIs —
// callers can't match on it.
// ============================================================================

// --- 3a: anyhow::Result — implicit error wrapping --------------------------

// Any error type converts into anyhow::Error automatically via ?
fn read_config(path: &str) -> Result<String> {
    let contents = fs::read_to_string(path)?; // io::Error → anyhow::Error
    Ok(contents)
}

fn parse_port(s: &str) -> Result<u16> {
    let n: u32 = s.trim().parse()?;           // ParseIntError → anyhow::Error
    ensure!(n <= 65535, "port {n} is out of valid range 0–65535");
    Ok(n as u16)
}

// --- 3b: anyhow! macro — create an anyhow error from a string -------------

fn validate_username(name: &str) -> Result<()> {
    if name.is_empty() {
        // anyhow! builds an anyhow::Error from a format string — no type needed
        return Err(anyhow!("username cannot be empty"));
    }
    if name.len() > 32 {
        return Err(anyhow!("username '{}' exceeds 32 characters", name));
    }
    Ok(())
}

// --- 3c: bail! macro — anyhow! + return Err in one line -------------------

fn validate_age(age: i32) -> Result<()> {
    if age < 0 {
        bail!("age cannot be negative, got {age}");
    }
    if age > 150 {
        bail!("age {age} is implausibly large");
    }
    Ok(())
}

// --- 3d: ensure! macro — assert-like, returns Err on false ----------------

fn divide(a: f64, b: f64) -> Result<f64> {
    ensure!(b != 0.0, "cannot divide {a} by zero");
    Ok(a / b)
}

// ============================================================================
// SECTION 4 — .context() and .with_context()
//
// anyhow's killer feature: attach a message to any error without defining
// a new type. The original error becomes .source().
//
//   .context("msg")           — attaches a static message (always evaluated)
//   .with_context(|| "msg")   — attaches a lazily evaluated message
//                               (prefer this when formatting is involved)
// ============================================================================

fn load_port_from_file(path: &str) -> Result<u16> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("reading port file '{path}'"))?;  // lazy

    let port = raw.trim().parse::<u16>()
        .context("port file must contain a valid u16")?;           // static

    Ok(port)
}

fn initialise_app(config_path: &str) -> Result<()> {
    let _port = load_port_from_file(config_path)
        .context("initialising application")?; // adds another context layer

    Ok(())
}

// ============================================================================
// SECTION 5 — Mixing thiserror library errors with anyhow application code
//
// The common pattern in larger codebases:
//
//   lib.rs  — defines typed errors with thiserror
//   main.rs — calls library functions, uses anyhow to propagate and display
//
// Library errors convert into anyhow::Error via ? because they implement
// std::error::Error.
// ============================================================================

// Pretend this is in your library:
mod db {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum DbError {
        #[error("connection timed out after {secs}s")]
        Timeout { secs: u64 },

        #[error("record {id} not found")]
        NotFound { id: u64 },
    }

    pub fn fetch_user(id: u64) -> Result<String, DbError> {
        if id == 0 {
            return Err(DbError::Timeout { secs: 30 });
        }
        if id > 1000 {
            return Err(DbError::NotFound { id });
        }
        Ok(format!("user_{id}"))
    }
}

// Application code: uses anyhow, calls into library
fn get_username(id: u64) -> Result<String> {
    // DbError → anyhow::Error via ?, then .context() wraps it
    let user = db::fetch_user(id)
        .with_context(|| format!("fetching user {id} from database"))?;
    Ok(user)
}

// When you need to match on library errors from anyhow::Error, use .downcast_ref:
fn handle_db_error(id: u64) {
    match get_username(id) {
        Ok(u) => println!("got: {u}"),
        Err(e) => {
            // Downcast back to the concrete library error for recovery logic
            if let Some(db_err) = e.downcast_ref::<db::DbError>() {
                match db_err {
                    db::DbError::Timeout { secs } =>
                        eprintln!("timed out after {secs}s — will retry"),
                    db::DbError::NotFound { id } =>
                        eprintln!("no user with id {id}"),
                }
            } else {
                // Unknown error kind — just display the chain
                eprintln!("unexpected error: {e:#}");
            }
        }
    }
}

// ============================================================================
// SECTION 6 — anyhow error display formats
//
//   {e}    — displays only the top-level message
//   {e:#}  — displays the full chain: "msg: caused by: caused by: ..."
//   {e:?}  — Debug, includes backtrace when RUST_BACKTRACE=1
// ============================================================================

fn demo_display_formats() {
    let result: Result<u16> = "bad"
        .parse::<u16>()
        .context("parsing port")
        .context("loading config");

    if let Err(e) = result {
        println!("{{e}}   = {e}");       // "loading config"
        println!("{{e:#}} = {e:#}");     // "loading config: parsing port: invalid digit..."
    }
}

// ============================================================================
// SECTION 7 — anyhow in tests
//
// anyhow::Result is particularly handy in tests — no need to define a
// custom error type just to use ? in test functions.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Tests can return anyhow::Result — a failing ? fails the test.
    #[test]
    fn test_validate_age_ok() -> Result<()> {
        validate_age(25)?;
        Ok(())
    }

    #[test]
    fn test_validate_age_negative() {
        assert!(validate_age(-1).is_err());
    }

    #[test]
    fn test_divide_by_zero() {
        let err = divide(1.0, 0.0).unwrap_err();
        // Check the error message directly
        assert!(err.to_string().contains("cannot divide"));
    }

    #[test]
    fn test_db_not_found() {
        let result = get_username(9999);
        assert!(result.is_err());
        // Downcast to check the concrete error type in tests
        let err = result.unwrap_err();
        assert!(err.downcast_ref::<db::DbError>().is_some());
    }

    #[test]
    fn test_thiserror_display() {
        let e = FormatDemo::OutOfRange { min: 0, max: 10, value: 99 };
        assert_eq!(e.to_string(), "range [0, 10) violated by value 99");
    }

    #[test]
    fn test_thiserror_source_chain() {
        use std::error::Error;

        let inner = DatabaseError::ConnectionRefused {
            host: "localhost".to_string(),
            port: 5432,
        };
        let outer = ServiceError::Database(inner);

        // Verify the source chain links correctly
        assert!(outer.source().is_some());
        assert_eq!(
            outer.source().unwrap().to_string(),
            "connection refused on 'localhost:5432'"
        );
    }

    #[test]
    fn test_context_chain() {
        let result: Result<u16> = "bad"
            .parse::<u16>()
            .context("parsing port")
            .context("loading config");

        let err = result.unwrap_err();
        // Top-level message
        assert_eq!(err.to_string(), "loading config");
        // Full chain via {:#}
        let chain = format!("{err:#}");
        assert!(chain.contains("parsing port"));
        assert!(chain.contains("invalid digit"));
    }
}

// ============================================================================
// SECTION 8 — Summary: when to use what
//
//  ┌──────────────────────────────┬──────────────────────────────────────────┐
//  │ Situation                    │ Recommendation                           │
//  ├──────────────────────────────┼──────────────────────────────────────────┤
//  │ Library error type           │ thiserror enum/struct                    │
//  │ Callers need to match errors │ thiserror enum                           │
//  │ App / binary propagation     │ anyhow::Result + ? + .context()          │
//  │ Quick error from string      │ anyhow! / bail!                          │
//  │ Assertion-style guard        │ ensure!                                  │
//  │ Mixed lib+app errors         │ thiserror in lib, anyhow in main         │
//  │ Test functions               │ anyhow::Result (just use ?)              │
//  │ Need to recover by type      │ err.downcast_ref::<ConcreteError>()      │
//  └──────────────────────────────┴──────────────────────────────────────────┘
// ============================================================================

fn main() -> Result<()> {
    println!("=== Section 1: thiserror enum ===");
    match get_user(0) {
        Ok(u)  => println!("user: {u}"),
        Err(e) => {
            eprintln!("ServiceError: {e}");
            use std::error::Error;
            if let Some(src) = e.source() {
                eprintln!("  caused by: {src}");
            }
        }
    }

    println!("\n=== Section 3: anyhow basics ===");
    println!("validate_username(''): {}", validate_username("").unwrap_err());
    println!("validate_age(-5):      {}", validate_age(-5).unwrap_err());
    println!("divide(1,0):           {}", divide(1.0, 0.0).unwrap_err());

    println!("\n=== Section 4: context chains ===");
    if let Err(e) = initialise_app("nonexistent.conf") {
        // {e:#} prints the full context chain separated by ': '
        eprintln!("startup failed: {e:#}");
    }

    println!("\n=== Section 5: lib + app mixing ===");
    handle_db_error(0);    // timeout
    handle_db_error(9999); // not found
    handle_db_error(1);    // success

    println!("\n=== Section 6: display formats ===");
    demo_display_formats();

    Ok(())
}