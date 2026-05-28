// ============================================================================
// RAW RUST ERROR HANDLING
// Demonstrates every layer of manual error handling from first principles,
// with no external crates — only std.
// ============================================================================

use std::fmt;
use std::num::ParseIntError;
use std::fs;
use std::io;

// ============================================================================
// SECTION 1 — THE Error TRAIT
//
// Any type can be a Rust error if it implements:
//   - std::error::Error        (marker + optional source() chain)
//   - std::fmt::Display        (user-facing message)
//   - std::fmt::Debug          (derived; required by Error)
//
// std::error::Error requires:
//   fn source(&self) -> Option<&(dyn Error + 'static)>
//
// `source()` returns the underlying error that caused THIS error, forming
// a linked chain you can walk with a while loop (see print_error_chain below).
// ============================================================================

// --- 1a: A leaf error (no cause) -------------------------------------------

#[derive(Debug)]
pub struct NotFoundError {
    pub resource: String,
}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "resource not found: '{}'", self.resource)
    }
}

impl std::error::Error for NotFoundError {
    // Default source() returns None — this is a root-cause error.
}

// --- 1b: A wrapping error (has a cause) ------------------------------------

#[derive(Debug)]
pub struct ConfigError {
    pub path:   String,
    pub source: io::Error,   // concrete inner type stored by value
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to load config from '{}'", self.path)
    }
}

impl std::error::Error for ConfigError {
    // Expose the cause — callers can walk the chain via .source().
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

// Walks and prints the full error chain produced by .source().
fn print_error_chain(err: &dyn std::error::Error) {
    eprintln!("Error: {err}");
    let mut source = err.source();
    let mut depth  = 1;
    while let Some(cause) = source {
        eprintln!("  Caused by [{depth}]: {cause}");
        source = cause.source();
        depth += 1;
    }
}

// ============================================================================
// SECTION 2 — ENUM ERRORS (the idiomatic choice)
//
// One enum covers all error *kinds* a module can produce.
// Callers match on the variant to decide how to recover.
// ============================================================================

#[derive(Debug)]
pub enum AppError {
    // Wraps std errors by value — zero allocation.
    Io(io::Error),

    // Wraps another std error by value.
    Parse(ParseIntError),

    // Domain errors carry their own data.
    NotFound(String),
    Unauthorized { user: String, action: String },

    // A context variant: wraps an inner AppError with added information.
    // Box<> is used because AppError cannot contain itself by value
    // (would be infinitely sized — the compiler rejects it).
    Context { message: String, cause: Box<AppError> },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e)              => write!(f, "I/O error: {e}"),
            AppError::Parse(e)           => write!(f, "parse error: {e}"),
            AppError::NotFound(name)     => write!(f, "not found: {name}"),
            AppError::Unauthorized { user, action }
            => write!(f, "user '{user}' cannot '{action}'"),
            AppError::Context { message, .. }
            => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e)                   => Some(e),
            AppError::Parse(e)                => Some(e),
            AppError::Context { cause, .. }   => Some(cause.as_ref()),
            _                                 => None,
        }
    }
}

// --- From impls: enable the `?` operator to auto-convert ----------------

// `?` on an io::Error inside a fn returning Result<_, AppError> will call
// AppError::from(io_error) automatically.
impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self { AppError::Io(e) }
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self { AppError::Parse(e) }
}

// ============================================================================
// SECTION 3 — Result<T, E> COMBINATORS
//
// The standard library gives you a rich set of methods on Result so you
// rarely need to write `match` by hand.
// ============================================================================

fn demo_combinators() {
    // map: transform Ok value, leave Err untouched
    let doubled: Result<i32, String> = Ok(21).map(|n| n * 2);
    println!("map:          {:?}", doubled);          // Ok(42)

    // map_err: transform Err value, leave Ok untouched
    let mapped_err: Result<i32, String> =
        Err(42_i32).map_err(|e| format!("code {e}"));
    println!("map_err:      {:?}", mapped_err);       // Err("code 42")

    // and_then: chain fallible operations (flatMap)
    let chained: Result<i32, ParseIntError> =
        "10".parse::<i32>().and_then(|n| "5".parse::<i32>().map(|m| n + m));
    println!("and_then:     {:?}", chained);          // Ok(15)

    // or_else: recover from an error by trying something else
    let recovered: Result<i32, ParseIntError> =
        "bad".parse::<i32>().or_else(|_| "99".parse::<i32>());
    println!("or_else:      {:?}", recovered);        // Ok(99)

    // unwrap_or: provide a default on Err
    let value: i32 = "bad".parse::<i32>().unwrap_or(0);
    println!("unwrap_or:    {value}");                // 0

    // unwrap_or_else: compute the default lazily
    let value2: i32 = "bad".parse::<i32>().unwrap_or_else(|_| -1);
    println!("unwrap_or_else: {value2}");             // -1

    // ok: convert Result<T,E> to Option<T>, discarding the error
    let opt: Option<i32> = Ok::<i32, &str>(7).ok();
    println!("ok:           {:?}", opt);              // Some(7)

    // transpose: swap Result<Option<T>,E> into Option<Result<T,E>>
    let r: Result<Option<i32>, &str> = Ok(Some(1));
    let o: Option<Result<i32, &str>> = r.transpose();
    println!("transpose:    {:?}", o);                // Some(Ok(1))

    // collect: Vec of Results into Result of Vec — stops at first error
    let numbers: Result<Vec<i32>, _> =
        vec!["1", "2", "3"].iter().map(|s| s.parse::<i32>()).collect();
    println!("collect ok:   {:?}", numbers);          // Ok([1, 2, 3])

    let bad: Result<Vec<i32>, _> =
        vec!["1", "bad", "3"].iter().map(|s| s.parse::<i32>()).collect();
    println!("collect err:  {}", bad.is_err());       // true
}

// ============================================================================
// SECTION 4 — THE ? OPERATOR
//
// `expr?` desugars to:
//
//   match expr {
//       Ok(val)  => val,
//       Err(e)   => return Err(From::from(e)),
//   }
//
// It works in any fn returning Result (or Option). Chaining ? creates a
// natural early-return pipeline without nested match arms.
// ============================================================================

fn read_file(path: &str) -> Result<String, AppError> {
    // io::Error is auto-converted to AppError::Io via From impl.
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}

fn parse_port(s: &str) -> Result<u16, AppError> {
    // ParseIntError → AppError::Parse, then range check.
    let n: u32 = s.trim().parse::<u32>()?;
    if n > 65535 {
        return Err(AppError::NotFound(format!("port {n} out of range")));
    }
    Ok(n as u16)
}

fn load_port_from_file(path: &str) -> Result<u16, AppError> {
    // Two sequential ? — each propagates its own error kind.
    let raw  = read_file(path)?;
    let port = parse_port(&raw)?;
    Ok(port)
}

// ============================================================================
// SECTION 5 — ADDING CONTEXT MANUALLY
//
// The stdlib has no built-in `.context()` method (that's anyhow's killer
// feature). Here are the raw patterns for attaching context to errors.
// ============================================================================

// Pattern A: a free function that wraps any error in a Context variant.
fn with_context<E: Into<AppError>>(message: impl Into<String>, result: Result<(), E>)
                                   -> Result<(), AppError>
{
    result.map_err(|e| AppError::Context {
        message: message.into(),
        cause:   Box::new(e.into()),
    })
}

// Pattern B: an extension trait so you can call .context("...") directly
// on any Result whose Err implements Into<AppError>.
trait ResultExt<T> {
    fn context(self, msg: impl Into<String>) -> Result<T, AppError>;
}

impl<T, E: Into<AppError>> ResultExt<T> for Result<T, E> {
    fn context(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.map_err(|e| AppError::Context {
            message: msg.into(),
            cause:   Box::new(e.into()),
        })
    }
}

fn load_config(path: &str) -> Result<String, AppError> {
    // Without the trait: verbose
    fs::read_to_string(path)
        .context(format!("loading config from '{path}'"))
}

// ============================================================================
// SECTION 6 — Box<dyn Error>: ERASING THE ERROR TYPE
//
// When you don't care about matching on error variants — e.g. in main(),
// in tests, or in glue code — you can erase the type entirely.
//
//   Box<dyn Error>         — not Send/Sync
//   Box<dyn Error + Send + Sync>  — thread-safe, required by tokio etc.
//
// The tradeoff: you lose the ability to match on the variant, but you
// gain the ability to return ANY error without From conversions.
// ============================================================================

fn demo_boxed_error() -> Result<(), Box<dyn std::error::Error>> {
    let _s: i32   = "42".parse()?;     // ParseIntError — no From needed
    let _f: String = fs::read_to_string("does_not_exist.txt")
        .unwrap_or_default();
    Ok(())
}

// A common type alias seen in libraries and CLI tools:
type DynResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn flexible_parse(s: &str) -> DynResult<i32> {
    let n = s.trim().parse::<i32>()?;
    Ok(n * 2)
}

// ============================================================================
// SECTION 7 — MULTIPLE ERROR TYPES WITHOUT Box<dyn Error>
//
// When you need to own the error (for pattern matching, no heap) but
// multiple unrelated error types exist, use an enum. This is the same
// pattern as AppError above, scaled up.
// ============================================================================

#[derive(Debug)]
pub enum ParseConfigError {
    BadInt   { field: &'static str, source: ParseIntError },
    MissingField(&'static str),
}

impl fmt::Display for ParseConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseConfigError::BadInt { field, .. } =>
                write!(f, "field '{field}' is not a valid integer"),
            ParseConfigError::MissingField(field) =>
                write!(f, "required field '{field}' is missing"),
        }
    }
}

impl std::error::Error for ParseConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseConfigError::BadInt { source, .. } => Some(source),
            _                                        => None,
        }
    }
}

fn parse_config(input: &str) -> Result<(String, u16), ParseConfigError> {
    let mut host = None;
    let mut port = None;

    for line in input.lines() {
        let mut parts = line.splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some("host"), Some(v)) => host = Some(v.trim().to_string()),
            (Some("port"), Some(v)) => {
                port = Some(v.trim().parse::<u16>().map_err(|e| {
                    ParseConfigError::BadInt { field: "port", source: e }
                })?);
            }
            _ => {}
        }
    }

    Ok((
        host.ok_or(ParseConfigError::MissingField("host"))?,
        port.ok_or(ParseConfigError::MissingField("port"))?,
    ))
}

// ============================================================================
// SECTION 8 — OPTION AND ITS INTERACTION WITH Result
// ============================================================================

fn find_user(id: u32) -> Option<String> {
    if id == 1 { Some("alice".to_string()) } else { None }
}

fn demo_option_result_interop() {
    // ok_or / ok_or_else: Option → Result
    let user: Result<String, AppError> = find_user(99)
        .ok_or_else(|| AppError::NotFound("user 99".to_string()));
    println!("ok_or_else:   {}", user.is_err());

    // ?: works in fns returning Option too
    fn inner(id: u32) -> Option<String> {
        let name = find_user(id)?;           // ? on Option returns None early
        Some(format!("Hello, {name}!"))
    }
    println!("option ?:     {:?}", inner(1));
    println!("option ? miss:{:?}", inner(2));

    // and_then chains on Option
    let upper = find_user(1).map(|s| s.to_uppercase());
    println!("map on Some:  {:?}", upper);

    // flatten: Option<Option<T>> → Option<T>
    let nested: Option<Option<i32>> = Some(Some(42));
    println!("flatten:      {:?}", nested.flatten());
}

// ============================================================================
// SECTION 9 — PANIC vs Result
//
// Panics are NOT for recoverable errors. Use them only for:
//   - Programming bugs (index out of bounds, unwrap on None you "know" is Some)
//   - Unrecoverable state (OOM, corrupted invariant)
//
// Never use panic for user input or I/O errors.
// ============================================================================

fn demo_panic_variants() {
    // unwrap() — panics with the Debug repr of the Err
    let _: i32 = "42".parse().unwrap();

    // expect() — panics with YOUR message + the Debug repr (prefer this over unwrap)
    let _: i32 = "42".parse().expect("seed value must be an integer");

    // Direct panic — for invariant violations
    fn index_checked(v: &[i32], i: usize) -> i32 {
        if i >= v.len() {
            panic!("index {i} out of bounds for len {}", v.len());
        }
        v[i]
    }
    println!("index_checked: {}", index_checked(&[1, 2, 3], 1));

    // assert! / assert_eq! — for invariant checks (panic on failure)
    let x = 2 + 2;
    assert_eq!(x, 4, "arithmetic is broken");
}

// ============================================================================
// SECTION 10 — main() RETURNING Result
//
// main() can return Result<(), E> where E: Debug.
// Rust prints the error with {:?} and exits with code 1.
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Section 1: Error trait & source chain ===");
    let cfg_err = ConfigError {
        path:   "/etc/app.conf".to_string(),
        source: io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"),
    };
    print_error_chain(&cfg_err);

    println!("\n=== Section 3: Combinators ===");
    demo_combinators();

    println!("\n=== Section 4: ? operator ===");
    match load_port_from_file("nonexistent.conf") {
        Ok(port) => println!("port: {port}"),
        Err(AppError::Io(e)) => println!("io error: {e}"),
        Err(AppError::Parse(e)) => println!("parse error: {e}"),
        Err(e) => println!("other: {e}"),
    }

    println!("\n=== Section 5: Manual context ===");
    let result = load_config("nonexistent.toml");
    if let Err(ref e) = result {
        print_error_chain(e);
    }

    println!("\n=== Section 6: Box<dyn Error> ===");
    println!("flexible_parse('21'): {:?}", flexible_parse("21"));
    println!("flexible_parse('bad'): {}", flexible_parse("bad").is_err());

    println!("\n=== Section 7: Parse config ===");
    let cfg = "host = localhost\nport = 8080";
    println!("parse_config ok:  {:?}", parse_config(cfg));
    let bad = "host = localhost\nport = bad";
    match parse_config(bad) {
        Err(ParseConfigError::BadInt { field, source }) =>
            println!("bad int in '{field}': {source}"),
        _ => {}
    }

    println!("\n=== Section 8: Option ↔ Result ===");
    demo_option_result_interop();

    println!("\n=== Section 9: Panic variants (safe calls only) ===");
    demo_panic_variants();

    Ok(())
}