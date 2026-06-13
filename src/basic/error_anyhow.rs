// anyhow crate.
// The mental model: anyhow is a context-annotated error chain. You lose the ability to match on
// variants, but you gain a clean, readable error trail that tells you exactly what the program
// was trying to do at every layer when it failed.

// The core philosophy first
// anyhow gives you a single type — anyhow::Error — that can wrap any error. You trade matchability
// for ergonomics. This is why:
//
// thiserror → libraries (callers need to match and handle specific variants)
// anyhow → applications/binaries (you just want errors to propagate and display nicely)

use serde::{Deserialize, Serialize};
use crate::advance::marcros::UserId;

// Basic usage -- Result<T> and ?
// anyhow re-exports its own Result<T> alias so you don't need to write Result<T, anyhow::Error>
// everywhere:
fn anyhow_basics() {
    // Any error that implements std::error::Error is automatically wrappable via ?.
    // No #[from] needed — it's all implicit.
    use anyhow::Result; // type alias.

    fn read_number(path: &str) -> Result<i32> {
        let content = std::fs::read_to_string(path)?;  // io::Error auto-wrapped
        let n = content.trim().parse::<i32>()?;         // ParseIntError auto-wrapped
        Ok(n)
    }
}

// Adding context with .context() and .with_context()
// This is anyhow's killer feature. It lets you annotate errors with human-readable context as
// they propagate up the call stack:
fn error_context() {

    #[derive(Debug, Deserialize)]
    struct Config {}

    use anyhow::{Context, Result};

    // The error chain when printed looks like:
    // application startup failed
    //
    // Caused by:
    //     0: failed to read config file
    //     1: No such file or directory (os error 2)
    fn load_config(path: &str) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .context("failed to read config file")?;

        let config: Config = serde_json::from_str(&content)
            .context("config file is not valid JSON")?;

        Ok(config)
    }

    fn start_app() -> Result<()> {
        let config = load_config("app.json")
            .context("application startup failed")?;
        // ...
        Ok(())
    }

    struct User{
        id: u64,
    }

    fn db_query(id: u64) -> Result<u64> {
        Ok(id)
    }

    // .with_context() is the lazy version — use it when building the message is expensive or
    // needs runtime values:
    fn fetch_user(id: u64) -> Result<User> {
        let id = db_query(id)
            .with_context(|| format!("failed to fetch user id={id}"))?;
        // closure only runs if there's an error
        Ok(User{id})
    }
}

// Creating errors from scratch -- anyhow! and bail!
// When there's no underlying error to wrap, use the anyhow! macro to create one:
fn create_from_scratch() {
    use anyhow::{anyhow, bail, Result};

    fn validate_age(age: i32) -> Result<()> {
        // anyhow! creates an error value — use when you need to return it manually
        if age < 0 {
            return Err(anyhow!("age cannot be negative, got {age}"));
        }

        // bail! is shorthand for return Err(anyhow!(...))
        if age > 150 {
            bail!("age {age} is unrealistically large");
        }

        Ok(())
    }
}

// Conditional rising errors with ensure!
// ensure! is like bail! but with a condition — equivalent to if !condition { bail!(...) }:
fn conditional_rising_error() {
    use anyhow::{ensure, Result};

    fn divide(a: f64, b: f64) -> Result<f64> {
        ensure!(b != 0.0, "cannot divide {a} by zero");
        ensure!(a.is_finite(), "dividend must be finite, got {a}");
        Ok(a / b)
    }
}

// Layered call stacks -- context builds a chain.
// Each .context() adds a layer. The full chain is preserved and printed in order from outermost
// to innermost:
fn layered_call_stacks() {
    use anyhow::{Context, Result};

    struct ServerConfig {
        port: u16,
    }

    fn extract_field(raw: &str, field: &str) -> Result<String> {
        Ok("".to_owned())
    }

    fn parse_port(s: &str) -> Result<u16> {
        s.parse::<u16>()
            .with_context(|| format!("`{s}` is not a valid port number"))
    }

    fn load_server_config(path: &str) -> Result<ServerConfig> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("could not read `{path}`"))?;

        let port_str = extract_field(&raw, "port")
            .context("missing `port` field")?;

        let port = parse_port(&port_str)
            .context("invalid port in config")?;

        Ok(ServerConfig { port })
    }

    // printed output:
    // Caused by:
    //     0: invalid port in config
    //     1: `abc` is not a valid port number
    //     2: invalid digit found in string

    fn main() -> Result<()> {
        let cfg = load_server_config("server.toml")
            .context("failed to initialise server")?;
        Ok(())
    }
}

// downcasting -- recovering concrete error types.
// When you need to inspect the underlying error type (e.g. handle one specific case):
fn downcast() {
    use anyhow::Result;
    use std::io;

    #[derive(Debug)]
    struct MyCustomError{}
    impl std::fmt::Display for MyCustomError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("MyCustomError"))
        }
    }

    fn handle_result(result: Result<()>) {
        if let Err(e) = result {
            // Try to downcast to a concrete type
            if let Some(io_err) = e.downcast_ref::<io::Error>() {
                match io_err.kind() {
                    io::ErrorKind::NotFound     => eprintln!("file not found"),
                    io::ErrorKind::PermissionDenied => eprintln!("permission denied"),
                    _ => eprintln!("io error: {io_err}"),
                }
                return;
            }

            // downcast (consuming) — takes ownership and returns Result<T>
            match e.downcast::<MyCustomError>() {
                Ok(custom) => eprintln!("custom error: {custom}"),
                Err(other)  => eprintln!("unknown error: {other}"),
            }
        }
    }
}

// Mixing thiserror + anyhow -- the recommended pattern.
fn mixing_in_workspace(){
    // --- mylib (library crate, uses thiserror) ---
    #[derive(Debug, thiserror::Error)]
    pub enum LibError {
        #[error("record {0} not found")]
        NotFound(u64),

        #[error("permission denied")]
        Forbidden,
    }

    // --- myapp (binary crate, uses anyhow) ---
    use anyhow::{Context, Result};
    use mylib::LibError;

    fn run(user_id: u64) -> Result<()> {
        mylib::fetch(user_id)
            .with_context(|| format!("could not load user {user_id}"))?;
        Ok(())
    }

    fn main() -> Result<()> {
        // Special case one variant, let anyhow handle everything else
        if let Err(e) = run(42) {
            if let Some(LibError::Forbidden) = e.downcast_ref::<LibError>() {
                eprintln!("access denied — check your permissions");
                std::process::exit(1);
            }
            return Err(e);
        }
        Ok(())
    }
}

// main() returning Result.
// anyhow::Result works directly as a main return type — errors are printed automatically:
use anyhow::{Context, Result};

// If it fails, Rust prints:
// Error: could not read input.txt
//
// Caused by:
//     No such file or directory (os error 2)
fn main() -> Result<()> {
    let n = std::fs::read_to_string("input.txt")
        .context("could not read input.txt")?
        .trim()
        .parse::<i32>()
        .context("input.txt does not contain a valid integer")?;

    println!("number: {n}");
    Ok(())
}