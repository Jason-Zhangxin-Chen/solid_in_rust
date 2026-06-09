
// Cow<T> (“Clone on Write”) is a smart pointer that holds either a borrowed reference (Borrowed)
// or an owned value (Owned). It lets you work with data that might be either borrowed or owned,
// and defers cloning until you actually need to mutate. When you call .to_mut() on a Cow, it will
// clone the borrowed data into an owned value only if it was borrowed – and then give you a mutable
// reference.
//
// Cow is not about shared ownership or interior mutability; it’s about lazy allocation and API
// flexibility. You use it when:
//
// You want to accept both &str and String in a function without duplicating code.
//
// You process data that may or may not need modification – and you only want to pay for a
// clone/allocation when mutation happens.
//
// You want to return a value that is sometimes a reference into some existing data, and sometimes
// a freshly allocated value.
//
// You want to avoid unnecessary allocations in performance‑sensitive paths.

fn process_text_on_demand() {
    use std::borrow::Cow;

    fn escape_html(input: &str) -> Cow<str> {
        if input.contains(&['<', '>', '&', '"', '\'']) {
            // Needs modification → allocate and build a new String
            let mut escaped = input.to_owned();
            escaped = escaped.replace("&", "&amp;");
            escaped = escaped.replace("<", "&lt;");
            escaped = escaped.replace(">", "&gt;");
            escaped = escaped.replace("\"", "&quot;");
            escaped = escaped.replace("'", "&#39;");
            Cow::Owned(escaped)
        } else {
            // No modification needed → just borrow the original
            Cow::Borrowed(input)
        }
    }

    let safe = escape_html("hello world");
    // safe is Cow::Borrowed("hello world") – no allocation.
    let dangerous = escape_html("<script>");
    // dangerous is Cow::Owned("&lt;script&gt;") – allocated only for this one.
}

// Function parameters that accept both &str and String gracefully.
// You can design a function that accepts impl Into<Cow<str>>, which lets callers pass a &str,
// a String, or even a Cow<str> directly.
fn cow_as_input() {
    use std::borrow::Cow;

    // Below spec means: text is some concrete type T that can be converted into a Cow<'static, str>
    // See the banket impl in std:
    // impl<T, U> Into<U> for T
    // where
    //     U: From<T>,
    // a type T satisfies Into<Cow<'static, str>> if and only if Cow<'static, str> implements From<T>.
    //
    // The standard library provides these From implementations:
    //
    // impl<'a> From<&'a str> for Cow<'a, str> – works for any lifetime, including 'static.
    //
    // impl<'a> From<String> for Cow<'a, str> – works for any lifetime, including 'static.
    //
    // Therefore:
    //
    // A &'static str (like "hello") implements Into<Cow<'static, str>>.
    //
    // A String implements Into<Cow<'static, str>>.
    //
    // So yes, the direction is: Cow<'static, str> must implement From<&str> and From<String>.


    // Dispatch - static, no vtable.
    // impl Into<Cow<'static, str>> is syntactic sugar for a generic parameter. The compiler
    // monomorphises process_text for each concrete type passed (here &str and String), generating
    // separate function versions at compile time. This is pure static dispatch. There is no
    // dynamic dispatch (vtable) involved.
    //
    // The call text.into() is also resolved statically – the compiler knows exactly which
    // From/Into conversion to call. And Cow itself is a simple enum (Borrowed or Owned),
    // so all method calls like contains or to_mut are statically dispatched as well.

    fn process_text(text: impl Into<Cow<'static, str>>) {
        let mut text = text.into();
        // We can call .to_mut() if we need to modify;
        // if the caller passed a String, it's already owned.
        if text.contains("bad") {
            text.to_mut().push_str(" (cleaned)");
        }
        println!("Processed: {text}");
    }

    process_text("hello");                // borrowed &str → no allocation
    process_text(String::from("badstuff")); // String → moves the existing allocation
}

// Deserialization / Parsing: borrow when possible, own when necessary.
// Cow<str> (and Cow<[u8]>) is used extensively in crates like serde_json. When parsing JSON, if
// a string contains no escapes, you can borrow it from the input buffer. If it contains
// escapes (like \n), you need to allocate a new String. Cow<str> perfectly represents this
// “borrow if possible, own otherwise” result.

fn borrow_or_to_own() {
    use std::borrow::Cow;

    fn parse_string_literal(raw: &str) -> Cow<str> {
        // In a real parser, this avoids allocating memory for the vast majority of strings that
        // are plain text, while still supporting escaped sequences transparently.
        // Suppose raw is the inside of quotes, e.g. "hello" or "line1\nline2"
        if raw.contains('\\') {
            // Contains escapes → we need to process and allocate
            let mut owned = String::new();
            // (simplified escape handling)
            for ch in raw.chars() {
                if ch == '\\' { /* handle escape */ }
                else { owned.push(ch); }
            }
            Cow::Owned(owned)
        } else {
            // No escapes → borrow the slice
            Cow::Borrowed(raw)
        }
    }
}

// Returning a value that is either a reference or freshly created.
// Sometimes a function needs to return data that sometimes comes from a static cache and sometimes
// is built at runtime. Cow lets you unify the return type.
fn returning_cow_value() {
    use std::borrow::Cow;

    static GREETING: &str = "Hello, world!";
    // This is exactly what std::str::from_utf8_lossy does: it returns a Cow<str> – borrowing the
    // input bytes if they’re valid UTF-8, or allocating a new String with replacement characters
    // otherwise.
    fn get_greeting(name: Option<&str>) -> Cow<'static, str> {
        match name {
            Some(n) => Cow::Owned(format!("Hello, {n}!")),  // dynamically allocated
            None => Cow::Borrowed(GREETING),                      // static reference
        }
    }

    let a = get_greeting(None);
    let b = get_greeting(Some("Alice"));
    // Both are Cow<'static, str>, usable uniformly.
}

// Working with Vec<T> / [T]: avoid unnecessary cloning of owned data.
// If you have a function that needs an owned Vec<T> but sometimes receives a Vec<T> that can just
// be moved, and sometimes a &[T] that must be cloned, use Cow<[T]>. This avoids copying the vector
// when ownership is already available.
fn avoid_clone() {
    use std::borrow::Cow;

    fn ensure_owned(data: Cow<[i32]>) -> Vec<i32> {
        // If data is owned, this just returns the Vec.
        // If borrowed, it clones into a new Vec.
        data.into_owned()
    }

    let borrowed = [1, 2, 3];
    let owned = vec![4, 5, 6];

    let v1 = ensure_owned(Cow::Borrowed(&borrowed)); // clones
    let v2 = ensure_owned(Cow::Owned(owned));        // no clone, moves the Vec
}