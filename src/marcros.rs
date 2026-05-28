// =============================================================================
// COMPREHENSIVE RUST MACROS GUIDE
// =============================================================================
// This file covers every major part of Rust macro syntax, organized into
// themed sections. Each section is a standalone "demo" function you can call.
//
// TWO TYPES OF MACROS IN RUST:
//   1. Declarative Macros  — `macro_rules!`  (covered fully here)
//   2. Procedural Macros   — derive/attribute/function-like (notes at bottom)
//
// Run this file with:  rustc rust_macros_guide.rs && ./rust_macros_guide
// =============================================================================

fn main() {
    demo_01_basic_syntax();
    demo_02_fragment_specifiers();
    demo_03_repetition();
    demo_04_multiple_arms();
    demo_05_nested_repetition();
    demo_06_recursive_macros();
    demo_07_tt_munching();
    demo_08_macro_as_expression();
    demo_09_creating_structs_and_impls();
    demo_10_variadic_macros();
    demo_11_scoping_and_export();
    demo_12_macro_hygiene();
    demo_13_debugging_macros();
    println!("\nAll macro demos completed.");
}

// =============================================================================
// 01 — BASIC SYNTAX
// The anatomy of a macro_rules! macro.
// =============================================================================
//
//  macro_rules! <name> {
//      (<pattern>) => { <expansion> };
//      (<pattern>) => { <expansion> };   // multiple arms allowed
//  }
//
//  DELIMITERS: () [] {} are all valid for both the pattern and expansion.
//  By convention:  () for calls,  {} for blocks,  [] for lists.

macro_rules! say_hello {
    // Empty pattern — called as say_hello!()
    () => {
        println!("Hello, world!");
    };
}

macro_rules! greet {
    // $name captures one expression fragment
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

fn demo_01_basic_syntax() {
    println!("\n=== 01: Basic Syntax ===");

    say_hello!();           // expands to: println!("Hello, world!");
    greet!("Alice");        // expands to: println!("Hello, Alice!");
    greet!(42);             // any expr works — even numbers
}

// =============================================================================
// 02 — FRAGMENT SPECIFIERS
// These are the "types" of things a macro pattern can capture.
//
//   $name:expr     — any expression:  1+2, foo(), "str", true
//   $name:ident    — an identifier:   my_var, MyStruct, foo
//   $name:ty       — a type:          i32, Vec<u8>, Option<String>
//   $name:pat      — a pattern:       Some(x), (a, b), 42
//   $name:stmt     — a statement:     let x = 5;  or  x += 1;
//   $name:block    — a block:         { let x = 1; x + 2 }
//   $name:item     — a top-level item: fn, struct, impl, use ...
//   $name:meta     — attribute content: derive(Debug) or cfg(test)
//   $name:tt       — a single token tree — the most flexible, matches almost anything
//   $name:literal  — a literal only:   42, "str", 3.14, b'x'
//   $name:lifetime — a lifetime:       'a, 'static
//   $name:vis      — a visibility:     pub, pub(crate), (empty)
//   $name:path     — a path:           std::vec::Vec, crate::MyType
// =============================================================================

// :ident — capture an identifier to use as a variable name
macro_rules! declare_var {
    ($name:ident, $val:expr) => {
        let $name = $val;
    };
}

// :ty — capture a type, useful for generic-like macros
macro_rules! default_value {
    ($t:ty) => {
        <$t>::default()
    };
}

// :block — capture a whole block of code
macro_rules! time_it {
    ($label:expr, $body:block) => {{
        let start = std::time::Instant::now();
        let result = $body;
        println!("[{}] took {:?}", $label, start.elapsed());
        result
    }};
}

// :pat — capture a pattern, useful in match-like macros
macro_rules! assert_matches {
    ($val:expr, $pat:pat) => {
        match $val {
            $pat => println!("  matched pattern ok"),
            _    => panic!("  value did not match pattern"),
        }
    };
}

// :literal — only literals, not arbitrary expressions
macro_rules! repeat_str {
    ($s:literal, $n:expr) => {
        $s.repeat($n)
    };
}

// :tt — single token tree (most permissive)
// Useful when you don't know exactly what will be passed
macro_rules! print_token {
    ($t:tt) => {
        println!("  token: {}", stringify!($t));
    };
}

fn demo_02_fragment_specifiers() {
    println!("\n=== 02: Fragment Specifiers ===");

    // :ident
    declare_var!(score, 100);
    println!("  score = {}", score);

    // :ty
    let zero_int: i32 = default_value!(i32);
    let zero_str: String = default_value!(String);
    println!("  i32 default = {}", zero_int);
    println!("  String default = {:?}", zero_str);

    // :block
    let result = time_it!("computation", {
        let mut sum = 0u64;
        for i in 0..1000 { sum += i; }
        sum
    });
    println!("  result = {}", result);

    // :pat
    assert_matches!(Some(42), Some(_));

    // :literal
    let s = repeat_str!("ab", 3);
    println!("  repeated = {}", s);

    // :tt
    print_token!(hello);
    print_token!(+);
    print_token!(42);
}

// =============================================================================
// 03 — REPETITION
// Run a pattern zero or more times.
//
//   $( <pattern> )*    — zero or more  (Kleene star)
//   $( <pattern> )+    — one  or more  (Kleene plus)
//   $( <pattern> )?    — zero or one   (optional)
//
// A separator token can go between the repetition and the quantifier:
//   $( $x:expr ),*     — comma-separated zero or more
//   $( $x:expr );+     — semicolon-separated one or more
// =============================================================================

// Accepts any number of arguments (like println!'s args)
macro_rules! sum {
    ( $( $x:expr ),* ) => {{
        let mut total = 0i64;
        $(
            total += $x as i64;    // $x is expanded once per repetition
        )*
        total
    }};
}

// Optional argument with ?
macro_rules! greet_optional {
    ($name:expr $(, $title:expr)?) => {
        // $(...)? expands to nothing if the optional part wasn't provided
        println!("  Hello, {}{}", $( concat!($title, " ") ,)? $name);
    };
}

// Build a Vec from arguments — this is essentially how vec![] works internally
macro_rules! my_vec {
    ( $( $elem:expr ),* ) => {{
        let mut v = Vec::new();
        $(
            v.push($elem);
        )*
        v
    }};

    // Trailing comma variant
    ( $( $elem:expr ),+ , ) => {
        my_vec![ $($elem),* ]
    };
}

// One-or-more with + quantifier
macro_rules! print_all {
    ( $first:expr $(, $rest:expr)+ ) => {
        print!("  {} ", $first);
        $(
            print!("{} ", $rest);
        )+
        println!();
    };
}

fn demo_03_repetition() {
    println!("\n=== 03: Repetition ===");

    let s = sum!(1, 2, 3, 4, 5);
    println!("  sum(1..5) = {}", s);

    let s_empty = sum!(0);       // zero repetitions — total stays 0
    println!("  sum() = {}", s_empty);

    greet_optional!("Alice", "");
    greet_optional!("Bob", "Dr.");

    let v: Vec<i32> = my_vec![10, 20, 30];
    println!("  my_vec = {:?}", v);

    print_all!("one", "two", "three");
}

// =============================================================================
// 04 — MULTIPLE MATCH ARMS
// Macros can have many arms, matched top-to-bottom like match statements.
// This is how you build overloaded or context-sensitive macros.
// =============================================================================

macro_rules! log {
    // Arm 1: just a message
    ($msg:expr) => {
        println!("[INFO] {}", $msg);
    };

    // Arm 2: level + message
    ($level:ident, $msg:expr) => {
        println!("[{}] {}", stringify!($level), $msg);
    };

    // Arm 3: level + format string + args (like println!)
    ($level:ident, $fmt:literal, $($arg:expr),*) => {
        println!(concat!("[{}] ", $fmt), stringify!($level), $($arg),*);
    };
}

macro_rules! op {
    // Different operations dispatched by keyword token
    (add $a:expr, $b:expr)  => { $a + $b };
    (sub $a:expr, $b:expr)  => { $a - $b };
    (mul $a:expr, $b:expr)  => { $a * $b };
    (max $a:expr, $b:expr)  => { if $a > $b { $a } else { $b } };
}

fn demo_04_multiple_arms() {
    println!("\n=== 04: Multiple Arms ===");

    log!("server started");
    log!(WARN, "disk usage high");
    log!(ERROR, "connection failed after {} retries", 3);

    println!("  add 3+4 = {}", op!(add 3, 4));
    println!("  sub 9-2 = {}", op!(sub 9, 2));
    println!("  max 5,8 = {}", op!(max 5, 8));
}

// =============================================================================
// 05 — NESTED REPETITION
// Repetition groups can be nested. Each variable must be used at the
// correct nesting depth — a variable captured in depth 1 cannot be
// directly used in depth 2 without an enclosing repetition.
// =============================================================================

// Build a HashMap — note $key and $val are both at depth 1
macro_rules! hashmap {
    ( $( $key:expr => $val:expr ),* $(,)? ) => {{
        let mut map = ::std::collections::HashMap::new();
        $(
            map.insert($key, $val);
        )*
        map
    }};
}

// Matrix: nested repetition — outer rows, inner columns
macro_rules! matrix {
    (
        $( [ $( $elem:expr ),* ] ),*   // rows separated by commas
    ) => {
        vec![
            $(
                vec![ $($elem),* ],    // each row is a vec
            )*
        ]
    };
}

fn demo_05_nested_repetition() {
    println!("\n=== 05: Nested Repetition ===");

    let scores = hashmap! {
        "Alice" => 95,
        "Bob"   => 87,
        "Carol" => 92,   // trailing comma handled by $(,)?
    };
    let mut names: Vec<_> = scores.keys().collect();
    names.sort();
    for name in names {
        println!("  {} => {}", name, scores[name]);
    }

    let m = matrix!(
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9]
    );
    println!("  matrix[1][2] = {}", m[1][2]);   // 6
}

// =============================================================================
// 06 — RECURSIVE MACROS
// A macro can call itself. Rust limits recursion depth (default: 128).
// The pattern is: a base case arm + a recursive arm.
// =============================================================================

// Count arguments at compile time using recursion
macro_rules! count {
    ()                    => { 0usize };                   // base case
    ($head:tt $($tail:tt)*) => { 1 + count!($($tail)*) };  // recursive case
}

// Implement a trait for multiple types in one call
macro_rules! impl_display {
    // Base case: nothing left to implement
    () => {};

    // Recursive case: implement for first type, recurse on the rest
    ($t:ty $(, $rest:ty)*) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        impl_display!($($rest),*);   // recurse
    };
}

// Reverse a list of expressions using recursion + accumulator pattern
macro_rules! reverse {
    // When input is empty, emit the accumulator
    ( [] [$($acc:expr),*] ) => {
        vec![$($acc),*]
    };
    // Move head to front of accumulator, recurse on tail
    ( [$head:expr $(, $tail:expr)*] [$($acc:expr),*] ) => {
        reverse!([$($tail),*] [$head $(,$acc)*])
    };
    // Public entry point — starts with empty accumulator
    ( $($x:expr),* ) => {
        reverse!([$($x),*] [])
    };
}

fn demo_06_recursive_macros() {
    println!("\n=== 06: Recursive Macros ===");

    // count! resolves entirely at compile time
    const N: usize = count!(a b c d e);
    println!("  count of [a b c d e] = {}", N);

    let reversed = reverse!(1, 2, 3, 4, 5);
    println!("  reversed = {:?}", reversed);
}

// =============================================================================
// 07 — TOKEN TREE (TT) MUNCHING
// The most powerful declarative macro technique.
// :tt matches ANY single token or any (...)/{...}/[...] group.
// You "munch" through a stream of tokens one at a time.
// =============================================================================

// Parse a mini DSL: "if <cond> then <block> else <block>"
macro_rules! simple_if {
    (if $cond:expr => $then:block else $else:block) => {
        if $cond $then else $else
    };
}

// Build SQL-like SELECT statement string via TT munching
macro_rules! sql_select {
    // Final case: no more columns
    (@cols [] $table:ident) => {
        format!("SELECT * FROM {}", stringify!($table))
    };
    // One column
    (@cols [$col:ident] $table:ident) => {
        format!("SELECT {} FROM {}", stringify!($col), stringify!($table))
    };
    // Multiple columns — munch the first, recurse
    (@cols [$first:ident, $($rest:ident),+] $table:ident) => {{
        let rest = sql_select!(@cols [$($rest),+] $table);
        // Rebuild with first column prepended
        let without_from = rest.trim_end_matches(
            &format!(" FROM {}", stringify!($table))
        ).to_string();
        format!("SELECT {}, {} FROM {}",
            stringify!($first),
            without_from.trim_start_matches("SELECT "),
            stringify!($table))
    }};

    // Public entry points
    (SELECT * FROM $table:ident) => {
        sql_select!(@cols [] $table)
    };
    (SELECT $($col:ident),+ FROM $table:ident) => {
        sql_select!(@cols [$($col),+] $table)
    };
}

fn demo_07_tt_munching() {
    println!("\n=== 07: TT Munching ===");

    let x = 10;
    simple_if!(
        if x > 5 => {
            println!("  x is greater than 5");
        } else {
            println!("  x is not greater than 5");
        }
    );

    let q1 = sql_select!(SELECT * FROM users);
    let q2 = sql_select!(SELECT id, name FROM users);
    println!("  {}", q1);
    println!("  {}", q2);
}

// =============================================================================
// 08 — MACROS AS EXPRESSIONS
// The expansion of a macro can be a value. Using {{...}} in the expansion
// creates a block expression that evaluates to its last statement.
// =============================================================================

macro_rules! max_of {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {{
        let rest_max = max_of!($($rest),+);
        if $x > rest_max { $x } else { rest_max }
    }};
}

macro_rules! unwrap_or_return {
    ($expr:expr, $ret:expr) => {
        match $expr {
            Some(val) => val,
            None      => return $ret,
        }
    };
}

macro_rules! cfg_value {
    ($val:expr) => {{
        // Block expression: can contain let bindings, returns a value
        let v = $val;
        let doubled = v * 2;
        doubled + 1
    }};
}

fn demo_08_macro_as_expression() {
    println!("\n=== 08: Macros as Expressions ===");

    // max_of! returns a value directly
    let m = max_of!(3, 7, 2, 9, 1);
    println!("  max(3,7,2,9,1) = {}", m);

    // cfg_value! evaluates to a single value
    let v = cfg_value!(10);
    println!("  cfg_value(10) = {}", v);  // (10*2)+1 = 21

    // unwrap_or_return! used inside a function
    fn find_first(items: &[Option<i32>]) -> i32 {
        let first = unwrap_or_return!(items[0], -1);
        first * 10
    }
    println!("  find_first([None]) = {}", find_first(&[None]));
    println!("  find_first([Some(5)]) = {}", find_first(&[Some(5)]));
}

// =============================================================================
// 09 — GENERATING STRUCTS AND IMPLS
// Macros can emit items (structs, enums, impls, functions).
// This is one of the most practical uses — reducing boilerplate.
// =============================================================================

// Generate a newtype wrapper with From/Into and Display
macro_rules! newtype {
    (
        $(#[$attr:meta])*      // optional attributes like #[derive(Debug)]
        $vis:vis               // visibility: pub / pub(crate) / nothing
        struct $name:ident
        ($inner:ty)            // the wrapped type
        $(; display: $fmt:literal)? // optional custom display format
    ) => {
        $(#[$attr])*
        $vis struct $name(pub $inner);

        impl From<$inner> for $name {
            fn from(v: $inner) -> Self { Self(v) }
        }

        impl From<$name> for $inner {
            fn from(n: $name) -> Self { n.0 }
        }

        impl std::ops::Deref for $name {
            type Target = $inner;
            fn deref(&self) -> &Self::Target { &self.0 }
        }

        // Only emit Display if the user provided a format string
        $(
            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, $fmt, self.0)
                }
            }
        )?
    };
}

// Generate a simple builder pattern (setter methods on a struct)
// NOTE: Generating a *separate* BuilderStruct name from $name requires
// the `paste` crate ([<$name Builder>] syntax). Here we generate setters
// directly on the struct to stay dependency-free.
macro_rules! with_setters {
    (
        #[derive($($der:ident),*)]
        $vis:vis struct $name:ident {
            $( $field:ident : $ty:ty = $default:expr ),* $(,)?
        }
    ) => {
        #[derive($($der),*)]
        $vis struct $name {
            $( pub $field: $ty, )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $( $field: $default, )*
                }
            }
        }

        impl $name {
            // Generate a chainable setter: .field(val) -> Self
            $(
                pub fn $field(mut self, val: $ty) -> Self {
                    self.$field = val;
                    self
                }
            )*
        }
    };
}

newtype!(
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserId(u64);
    display: "user#{}"
);

newtype!(
    #[derive(Debug, Clone)]
    pub struct Email(String)
);

with_setters! {
    #[derive(Debug, Clone)]
    pub struct ServerConfig {
        host: String = "localhost".to_string(),
        port: u16    = 8080,
        workers: u8  = 4,
    }
}

fn demo_09_creating_structs_and_impls() {
    /*
    println!("\n=== 09: Generating Structs and Impls ===");

    let id = UserId::from(42u64);
    println!("  UserId: {:?}", id);
    println!("  Display: {}", id);   // "user#42"
    println!("  Inner: {}", *id);

    let email = Email::from("alice@example.com".to_string());
    println!("  Email inner: {}", *email);

    // with_setters! gave ServerConfig a Default + chainable setters
    let cfg = ServerConfig::default()
        .host("0.0.0.0".to_string())
        .port(9000)
        .workers(8);
    println!("  ServerConfig: {:?}", cfg);

     */
}

// =============================================================================
// 10 — VARIADIC / OVERLOADED MACROS
// Simulate function overloading by matching different argument patterns.
// The key: order your arms from most specific to least specific.
// =============================================================================

macro_rules! connect {
    // No port — default to 8080
    ($host:expr) => {
        connect!($host, 8080)
    };

    // Host + port
    ($host:expr, $port:expr) => {
        connect!($host, $port, timeout = 30)
    };

    // Host + port + timeout keyword argument
    ($host:expr, $port:expr, timeout = $t:expr) => {
        format!("tcp://{}:{}  (timeout={}s)", $host, $port, $t)
    };
}

// Accept either a single value OR key=value pairs
macro_rules! config {
    // Single raw value
    ($val:expr) => {
        println!("  config value: {}", $val)
    };

    // Key=value pairs (one or more)
    ( $( $key:ident = $val:expr ),+ ) => {
        $(
            println!("  config[{}] = {}", stringify!($key), $val);
        )+
    };
}

fn demo_10_variadic_macros() {
    println!("\n=== 10: Variadic / Overloaded Macros ===");

    println!("  {}", connect!("localhost"));
    println!("  {}", connect!("db.internal", 5432));
    println!("  {}", connect!("cache.internal", 6379, timeout = 5));

    config!("simple value");
    config!(host = "localhost", port = 8080, debug = true);
}

// =============================================================================
// 11 — SCOPING AND EXPORTING
//
//   By default, macro_rules! macros are LOCAL to the module they're defined in.
//
//   To use a macro in a child module:
//     #[macro_use]  on the parent module declaration
//
//   To export a macro from your CRATE so others can use it:
//     #[macro_export]  — places it at the crate root
//
//   After Rust 2018, you can also use macros like items:
//     use my_crate::my_macro;
// =============================================================================

// This macro is only visible in this file / module
macro_rules! private_macro {
    () => { println!("  I am module-local") };
}

// #[macro_export] would make this available as `crate::public_macro!`
// Shown here without the attribute since we're in a single-file example:
macro_rules! public_macro {
    () => { println!("  I would be crate-public with #[macro_export]") };
}

mod child_module {
    // Without #[macro_use] on `mod child_module` in the parent,
    // macros defined outside are not automatically available here.
    // In a multi-file project you'd write:
    //   #[macro_use]
    //   mod child_module;
    pub fn run() {
        println!("  child_module::run() — macros scope is per-module");
    }
}

fn demo_11_scoping_and_export() {
    println!("\n=== 11: Scoping and Export ===");
    private_macro!();
    public_macro!();
    child_module::run();
    println!("  (See comments for #[macro_export] and #[macro_use] usage)");
}

// =============================================================================
// 12 — MACRO HYGIENE
//
// Rust macros are HYGIENIC: variables introduced inside a macro expansion
// do not clash with variables in the caller's scope.
//
// This is different from C macros (#define), which are text substitutions
// and can silently shadow the caller's variables.
// =============================================================================

macro_rules! hygienic_swap {
    ($a:expr, $b:expr) => {{
        // `temp` here lives in the macro's own hygiene context.
        // It will NOT collide with any `temp` the caller has.
        let temp = $a;
        $a = $b;
        $b = temp;
    }};
}

macro_rules! count_down {
    ($n:expr) => {{
        // `i` here is hygienic — won't touch caller's `i`
        let mut i = $n;
        while i > 0 {
            print!("{} ", i);
            i -= 1;
        }
        println!();
    }};
}

// To INTENTIONALLY break hygiene (e.g., introduce a variable into
// caller scope), you must receive the name as an :ident argument:
macro_rules! let_named {
    ($name:ident = $val:expr) => {
        let $name = $val;   // $name comes FROM the caller, so it's in caller scope
    };
}

fn demo_12_macro_hygiene() {
    println!("\n=== 12: Macro Hygiene ===");

    let mut x = 10;
    let mut y = 20;
    let temp = 999; // our own `temp` — hygienic_swap won't touch it

    hygienic_swap!(x, y);
    println!("  after swap: x={}, y={}", x, y);
    println!("  caller's temp is still: {}", temp); // still 999

    print!("  countdown: ");
    let i = 42; // our own `i`
    count_down!(5);
    println!("  caller's i is still: {}", i); // still 42

    // Breaking hygiene intentionally — let_named! injects `answer` into our scope
    let_named!(answer = 6 * 7);
    println!("  answer injected by macro: {}", answer);
}

// =============================================================================
// 13 — DEBUGGING MACROS
//
// Essential tools when your macro doesn't expand how you expect:
//
//   stringify!($tokens)       — converts tokens to a string literal (no eval)
//   concat!(...)              — concatenates literals at compile time
//   file!()                   — current source file name
//   line!()                   — current line number
//   column!()                 — current column
//   module_path!()            — current module path
//   env!("VAR")               — reads env variable at compile time
//   option_env!("VAR")        — Option version of env!
//
// For full expansion traces, use:
//   cargo expand              — (install: cargo install cargo-expand)
//   rustc -Zunpretty=expanded (nightly only)
// =============================================================================

macro_rules! debug_tokens {
    ($($t:tt)*) => {
        println!("  tokens: {}", stringify!($($t)*));
    };
}

macro_rules! where_am_i {
    () => {
        println!(
            "  called from {}:{}:{} in module '{}'",
            file!(), line!(), column!(), module_path!()
        );
    };
}

macro_rules! build_name {
    ($prefix:literal, $suffix:literal) => {
        concat!($prefix, "_", $suffix)   // compile-time string concatenation
    };
}

// dbg! is a built-in macro — shows file, line, and value
macro_rules! trace_val {
    ($e:expr) => {{
        let v = $e;
        println!("  trace [{}:{}] {} = {:?}", file!(), line!(), stringify!($e), v);
        v
    }};
}

fn demo_13_debugging_macros() {
    println!("\n=== 13: Debugging Macros ===");

    // stringify! — does NOT evaluate, just turns tokens into a string
    debug_tokens!(1 + 2 * 3);             // prints "1 + 2 * 3"
    debug_tokens!(let x: Vec<i32> = vec![1,2,3]);

    where_am_i!();

    const HANDLER_NAME: &str = build_name!("on", "click");
    println!("  concat name: {}", HANDLER_NAME);

    let v = trace_val!(2_u32.pow(10));
    println!("  computed: {}", v);

    // Built-in: dbg! returns the value so you can embed it in expressions
    let _result = dbg!(1 + 1) * dbg!(2 + 2);
}

// =============================================================================
// APPENDIX: PROCEDURAL MACROS (overview — these require a separate crate)
// =============================================================================
//
// Procedural macros are functions that take a TokenStream and return a
// TokenStream. They MUST live in a crate with `proc-macro = true` in
// Cargo.toml. There are three kinds:
//
// 1. CUSTOM DERIVE
//    #[proc_macro_derive(MyTrait)]
//    pub fn my_derive(input: TokenStream) -> TokenStream { ... }
//    Usage: #[derive(MyTrait)]  on a struct/enum
//
// 2. ATTRIBUTE MACROS
//    #[proc_macro_attribute]
//    pub fn my_attr(attr: TokenStream, item: TokenStream) -> TokenStream { ... }
//    Usage: #[my_attr(args)]  on any item
//
// 3. FUNCTION-LIKE MACROS (look like macro_rules but are procedural)
//    #[proc_macro]
//    pub fn my_macro(input: TokenStream) -> TokenStream { ... }
//    Usage: my_macro!(...)
//
// Key crates for writing proc macros:
//   syn       — parse Rust syntax from TokenStream
//   quote     — generate Rust code as TokenStream via quasi-quoting
//   proc-macro2 — TokenStream that works in both proc-macro and test contexts
//
// Example proc macro crate structure:
//
//   my-derive/
//   ├── Cargo.toml        ← [lib] proc-macro = true
//   └── src/lib.rs
//       use proc_macro::TokenStream;
//       use quote::quote;
//       use syn::{parse_macro_input, DeriveInput};
//
//       #[proc_macro_derive(HelloWorld)]
//       pub fn hello_world_derive(input: TokenStream) -> TokenStream {
//           let ast = parse_macro_input!(input as DeriveInput);
//           let name = &ast.ident;
//           quote! {
//               impl HelloWorld for #name {
//                   fn hello() { println!("Hello from {}!", stringify!(#name)); }
//               }
//           }.into()
//       }
//
// =============================================================================

// =====================================================================
//  Rust Macros – All Examples in One File
//  =====================================================================
//  This file demonstrates declarative macros (macro_rules!) and,
//  in a comment block at the end, procedural macros (attribute, derive,
//  function-like). Procedural macros must live in a separate crate
//  with `proc-macro = true`.
//  Every line of every macro definition is commented.
// =====================================================================

// ---------------------------------------------------------------------
// 1. Declarative Macros (macro_rules!)
// ---------------------------------------------------------------------

// 1.1 A simple macro – no arguments
macro_rules! say_hello {
    // Pattern: empty `()` triggers this arm.
    () => {
        // The macro expands to this exact print statement.
        println!("Hello, world!");
    };
}

// 1.2 Macro with a single expression argument
macro_rules! square {
    // Capture any Rust expression into meta-variable $x.
    ($x:expr) => {
        // Expansion: $x multiplied by $x.
        $x * $x
    };
}

// 1.3 Multiple patterns (arms)
macro_rules! print_message {
    // First arm: exactly one expression.
    ($msg:expr) => {
        // Print the single message.
        println!("Message: {}", $msg);
    };
    // Second arm: two expressions separated by comma.
    ($msg1:expr, $msg2:expr) => {
        // Print both messages with a separator.
        println!("Messages: {} | {}", $msg1, $msg2);
    };
}

// 1.4 Repetition – zero or more (*)
macro_rules! print_all {
    // $( ... )* repeats the inner pattern zero or more times.
    // Each repetition captures one expression into $elem.
    // The trailing comma means elements are separated by commas.
    ( $( $elem:expr ),* ) => {
        // In the expansion, $( ... );* repeats the print statement
        // for each captured $elem, separated by semicolons.
        $(
            println!("Element: {}", $elem);
        )*
    };
}

// 1.5 Repetition – one or more (+) and zero or one (?)
macro_rules! vec_min {
    // $first:expr is mandatory; $( , $rest:expr )+ captures one or more additional.
    ( $first:expr $( , $rest:expr )+ ) => {
        // We use a block so we can create a mutable Vec.
        {
            let mut v = Vec::new();
            v.push($first);            // Push the first element.
            $(
                v.push($rest);         // Push each additional element.
            )+
            v                          // Return the vector.
        }
    };
}

macro_rules! maybe_print {
    // $prefix:expr is mandatory; $( , $suffix:expr )? captures an optional second argument.
    ( $prefix:expr $( , $suffix:expr )? ) => {
        // Always print the prefix.
        println!("{}", $prefix);
        // Print the suffix only if it was provided. The $( )? expands exactly zero or one time.
        $(
            println!("{}", $suffix);
        )?
    };
}

// 1.6 Recursive macro (TT muncher)
macro_rules! sum {
    // Base case: a single expression – just return it.
    ($x:expr) => {
        $x
    };

    // sum!(1, 2, 3, 4) ===> (1 + (2 + (3+4)) = 10

    // Recursive case: first expression, then comma, then the rest as token trees ($(...)*).
    ($x:expr, $($rest:tt)*) => {
        // Add the first expression to the sum of the remaining token trees.
        $x + sum!($($rest)*)   // In recursion we drop the leading comma.
    };
}

// 1.7 Hygiene and $crate
// This simulates a helper function inside the same crate.
mod helpers {
    pub fn double(x: i32) -> i32 {
        x * 2
    }
}

// When exporting a macro, use $crate to refer to items in the defining crate.
#[macro_export]
macro_rules! call_double {
    // Capture an expression.
    ($val:expr) => {
        // $crate ensures the correct path even when used from another crate.
        $crate::marcros::helpers::double($val)
    };
}

#[cfg(test)]
mod tests {
    use crate::marcros::helpers;

#[test]
    fn test_macro() {
        // ----- Exercise all declarative macros -----
        say_hello!();                          // prints "Hello, world!"

        let sq = square!(3 + 2);              // expands to (3+2)*(3+2) = 25
        println!("square: {}", sq);

        print_message!("Hi");                 // "Message: Hi"
        print_message!("Hi", "there");        // "Messages: Hi | there"

        print_all!(1, 2.5, "three");          // prints three lines
        print_all!();                         // prints nothing (zero repetitions)

        let v = vec_min![10, 20, 30, 40];     // Vec with at least one element
        println!("vec_min: {:?}", v);

        maybe_print!("Only prefix");          // only "Only prefix"
        maybe_print!("Prefix", "And suffix"); // both lines

        let s = sum!(1, 2, 3, 4);            // expands to 1 + (2 + (3 + 4)) = 10
        println!("sum: {}", s);

        let d = call_double!(21);            // calls helpers::double
        println!("call_double: {}", d);
    }
}

// ---------------------------------------------------------------------
// 2. Procedural Macros
// ---------------------------------------------------------------------
// For these you must create a separate library crate with
// Cargo.toml:
//   [lib]
//   proc-macro = true
// and dependencies:
//   [dependencies]
//   syn = "2"
//   quote = "1"
//   proc-macro2 = "1"
//
// The code below is exactly what you would put in lib.rs of that crate.
// Every line of the macro definition functions is commented.
// ---------------------------------------------------------------------

/*
// In the proc-macro crate's lib.rs:

use proc_macro::TokenStream;                 // Import the compiler's token stream type.
use quote::quote;                            // The `quote!` macro for generating token streams.
use syn::{parse_macro_input, ItemFn, DeriveInput, LitInt}; // Parsing utilities.

// -------------------------------------------------------------------
// 2.1 Attribute-like macro
// -------------------------------------------------------------------

/// This attribute macro renames a function to its uppercase version.
#[proc_macro_attribute]                       // Tells the compiler this is an attribute macro.
pub fn rename_uppercase(
    _attr: TokenStream,                       // The tokens inside the attribute (we ignore them).
    item: TokenStream                         // The item the attribute is attached to (the function).
) -> TokenStream {
    // Parse the `item` token stream into a function syntax tree node.
    let input_fn = parse_macro_input!(item as ItemFn);
    // Extract the function's original identifier.
    let original_name = &input_fn.sig.ident;
    // Convert the identifier to an uppercase string and create a new identifier.
    let new_name = syn::Ident::new(
        &original_name.to_string().to_uppercase(),
        original_name.span(),
    );
    // Preserve the rest of the function (body, generics, etc.) as tokens.
    let rest = quote! { #input_fn };
    // Construct the output: the function with the new name and the original body.
    let output = quote! {
        fn #new_name() {
            #rest
        }
    };
    // Convert the proc_macro2 TokenStream into the compiler's TokenStream and return it.
    output.into()
}

// -------------------------------------------------------------------
// 2.2 Derive macro
// -------------------------------------------------------------------

/// Derives a `hello_world` method that prints the type name.
#[proc_macro_derive(HelloWorld)]             // Marks this as a derive macro with the name `HelloWorld`.
pub fn hello_world_derive(
    input: TokenStream                       // The item being derived on (e.g., a struct).
) -> TokenStream {
    // Parse the input into a DeriveInput syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    // Get the name of the type (for example, `MyStruct`).
    let name = &input.ident;
    // Generate the trait implementation tokens.
    let expanded = quote! {
        impl #name {
            pub fn hello_world() {
                // Use `stringify!` to turn the type name into a string at compile time.
                println!("Hello, World! My name is {}", stringify!(#name));
            }
        }
    };
    // Return the generated tokens.
    expanded.into()
}

// -------------------------------------------------------------------
// 2.3 Function-like macro
// -------------------------------------------------------------------

/// Takes two integer literals and returns their sum as a literal.
#[proc_macro]                                // This is a function-like macro.
pub fn add(
    input: TokenStream                       // The tokens inside the macro invocation parentheses.
) -> TokenStream {
    // Parse the input as a punctuated sequence of LitInt tokens separated by commas.
    let nums = parse_macro_input!(
        input with syn::punctuated::Punctuated::<LitInt, Token![,]>::parse_terminated
    );
    // Obtain an iterator over the literal integers.
    let mut iter = nums.iter();
    // Parse the first integer as i64.
    let first = iter.next()
        .expect("Need at least one integer")
        .base10_parse::<i64>()
        .unwrap();
    // Parse the second integer as i64.
    let second = iter.next()
        .expect("Need a second integer")
        .base10_parse::<i64>()
        .unwrap();
    let result = first + second;
    // Produce a single integer literal token for the sum.
    quote! { #result }.into()
}
*/

// ---------------------------------------------------------------------
// 3. Using procedural macros (in a different crate)
// ---------------------------------------------------------------------
// After building the proc-macro crate, you can use the macros like this
// in your main crate (Cargo.toml must list the proc-macro crate as a dependency):
//
// use my_proc_macro::rename_uppercase;
// use my_proc_macro::HelloWorld;
// use my_proc_macro::add;
//
// #[rename_uppercase]
// fn my_function() {
//     println!("Called from my_function");
// }
//
// #[derive(HelloWorld)]
// struct Demo;
//
// fn main() {
//     MY_FUNCTION();            // prints: Called from my_function
//     Demo::hello_world();      // prints: Hello, World! My name is Demo
//     let sum = add!(3, 7);     // expands to 10
//     println!("Sum: {}", sum);
// }