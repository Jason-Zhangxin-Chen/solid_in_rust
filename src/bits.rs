// bit_operations.rs
// Complete reference for every bitwise operation and helper in Rust.
// All methods shown on u32 unless noted — identical API exists on
// i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize.
// Compile: rustc --edition 2021 bit_operations.rs && ./bit_operations

#![allow(dead_code, unused_variables, clippy::all)]

fn main() {
    bitwise_operators();
    bit_counting();
    bit_shifting();
    bit_inspection();
    bit_manipulation();
    bit_conversion();
    byte_order();
    overflow_aware();
    bit_tricks();
    practical_examples();
}

// =============================================================================
// 1. Basic Bitwise Operators
// =============================================================================
fn bitwise_operators() {
    println!("\n===== Basic Operators =====");

    let a: u8 = 0b_1100_1010;   // 202
    let b: u8 = 0b_1010_1100;   // 172

    // AND — bit is 1 only if both are 1
    let and = a & b;             // 0b_1000_1000 = 136
    println!("AND:  {:08b} = {and}", and);

    // OR — bit is 1 if either is 1
    let or  = a | b;             // 0b_1110_1110 = 238
    println!("OR:   {:08b} = {or}", or);

    // XOR — bit is 1 if exactly one is 1 (toggle)
    let xor = a ^ b;             // 0b_0110_0110 = 102
    println!("XOR:  {:08b} = {xor}", xor);

    // NOT — flips every bit
    let not = !a;                // 0b_0011_0101 = 53
    println!("NOT:  {:08b} = {not}", not);

    // Left shift — multiply by 2^n (wraps on overflow in release, panics in debug)
    let shl = a << 2;            // 0b_0010_1000 = 40  (top bits dropped)
    println!("SHL:  {:08b} = {shl}", shl);

    // Right shift — divide by 2^n
    // u* types: logical shift (fills with 0)
    // i* types: arithmetic shift (fills with sign bit)
    let shr = a >> 2;            // 0b_0011_0010 = 50
    println!("SHR:  {:08b} = {shr}", shr);

    // Compound assignment versions
    let mut x = a;
    x &= b;   // x = x & b
    x |= b;   // x = x | b
    x ^= b;   // x = x ^ b
    x <<= 1;  // x = x << 1
    x >>= 1;  // x = x >> 1

    // Printing in different bases
    let n: u32 = 255;
    println!("decimal: {n}");
    println!("binary:  {n:b}");           // 11111111
    println!("binary:  {n:08b}");         // 00000000 (8 wide, zero-padded)
    println!("binary:  {n:#010b}");       // 0b00000000 (with prefix, 10 wide)
    println!("hex:     {n:x}");           // ff
    println!("hex:     {n:#010x}");       // 0x000000ff
    println!("octal:   {n:o}");           // 377
}

// =============================================================================
// 2. Bit Counting
// =============================================================================
fn bit_counting() {
    println!("\n===== Bit Counting =====");

    let n: u32 = 0b_0000_1011_0100_1101_0010_1100_0001_0110; // some u32

    // count_ones — population count / Hamming weight / "popcount"
    // Number of bits set to 1. Hardware instruction on most CPUs.
    let ones = n.count_ones();
    println!("count_ones:  {ones}");      // number of 1-bits

    // count_zeros — number of 0-bits (= bit_width - count_ones)
    let zeros = n.count_zeros();
    println!("count_zeros: {zeros}");     // 32 - ones

    // leading_zeros — count of 0-bits before the first 1 from MSB
    // USE: fast integer log2 / finding the highest set bit
    let lz = n.leading_zeros();
    println!("leading_zeros:  {lz}");

    // leading_ones — count of 1-bits before the first 0 from MSB
    let lo = u32::MAX.leading_ones();    // 32
    println!("leading_ones:   {lo}");

    // trailing_zeros — count of 0-bits after the last 1 from LSB
    // USE: finding the lowest set bit, fast modulo by power of 2
    let tz = n.trailing_zeros();
    println!("trailing_zeros: {tz}");

    // trailing_ones — count of 1-bits after the last 0 from LSB
    let to = 0b0111u32.trailing_ones();  // 3
    println!("trailing_ones:  {to}");

    // Practical: integer log2 (floor) for any non-zero value
    let x: u32 = 100;
    let log2_floor = u32::BITS - 1 - x.leading_zeros();
    println!("floor(log2(100)) = {log2_floor}"); // 6  (2^6=64 ≤ 100 < 128=2^7)

    // Practical: check if power of two
    let is_pow2 = x.count_ones() == 1;
    println!("100 is power of 2: {is_pow2}");    // false
    println!("128 is power of 2: {}", 128u32.count_ones() == 1); // true
}

// =============================================================================
// 3. Bit Shifting
// =============================================================================
fn bit_shifting() {
    println!("\n===== Bit Shifting =====");

    let n: u32 = 1;

    // Regular shift — panics in debug on oversized shift, wraps in release
    let shifted = n << 4;        // 16

    // Checked shift — returns None if shift >= bit width
    let r = n.checked_shl(4);    // Some(16)
    let r = n.checked_shl(32);   // None — shift too large
    let r = n.checked_shr(4);    // Some(0)
    let r = n.checked_shr(32);   // None

    // Wrapping shift — masks shift amount to valid range (shift % bit_width)
    let r = n.wrapping_shl(33);  // same as n << 1 (33 % 32 = 1)
    let r = n.wrapping_shr(33);  // same as n >> 1

    // Overflowing shift — (result, did_overflow: bool)
    let (r, overflow) = n.overflowing_shl(33); // (2, true)
    let (r, overflow) = n.overflowing_shr(4);  // (0, false)

    // rotate_left / rotate_right — bits wrap around instead of being dropped
    // Essential for cryptography (SHA, AES, etc.)
    let n: u32 = 0b_1000_0000_0000_0000_0000_0000_0000_0001;
    let rotl = n.rotate_left(1);
    //  before:  1000_0000_0000_0000_0000_0000_0000_0001
    //  after:   0000_0000_0000_0000_0000_0000_0000_0011
    //           MSB wrapped to LSB position
    println!("rotate_left(1):  {rotl:#034b}");

    let rotr = n.rotate_right(1);
    //  before:  1000_0000_0000_0000_0000_0000_0000_0001
    //  after:   1100_0000_0000_0000_0000_0000_0000_0000
    println!("rotate_right(1): {rotr:#034b}");

    // Multiply / divide by power of 2 via shift
    let x: u32 = 10;
    let times8  = x << 3;       // 80  (x * 2^3)
    let div4    = x >> 2;       // 2   (x / 2^2, rounds toward zero)

    println!("10 << 3 = {times8}");
    println!("10 >> 2 = {div4}");
}

// =============================================================================
// 4. Bit Inspection
// =============================================================================
fn bit_inspection() {
    println!("\n===== Bit Inspection =====");

    let n: u32 = 0b_1010_1100;

    // Check if a specific bit is set (bit index 0 = LSB)
    fn is_bit_set(n: u32, pos: u32) -> bool {
        n & (1 << pos) != 0
    }
    println!("bit 2 set: {}", is_bit_set(n, 2));  // true  (0b...1_00)
    println!("bit 0 set: {}", is_bit_set(n, 0));  // false (0b...1_0_0)

    // Read a specific bit (returns 0 or 1)
    fn get_bit(n: u32, pos: u32) -> u32 {
        (n >> pos) & 1
    }
    println!("bit 3: {}", get_bit(n, 3));         // 1

    // Extract a bitfield: bits [lo..=hi]
    fn get_bits(n: u32, lo: u32, hi: u32) -> u32 {
        let mask = (1u32 << (hi - lo + 1)) - 1;
        (n >> lo) & mask
    }
    // Extract bits [2..=5] of n = 0b_1010_1100
    //                                      ^^^^  bits 2-5
    println!("bits[2..=5]: {:04b}", get_bits(n, 2, 5)); // 1011

    // Check parity (even = 0, odd = 1)
    fn parity(n: u32) -> u32 {
        n.count_ones() & 1
    }
    println!("parity: {}", parity(n));             // 0 or 1

    // Bit width: minimum bits needed to represent the value
    let x: u32 = 100;
    let width = u32::BITS - x.leading_zeros();
    println!("bits needed for 100: {width}");      // 7

    // ilog2 / ilog10 — integer logarithm (Rust 1.67+, panics on 0)
    let x: u32 = 100;
    println!("ilog2(100):  {}", x.ilog2());        // 6
    println!("ilog10(100): {}", x.ilog10());       // 2
    println!("ilog(100,5): {}", x.ilog(5));        // 2 (5^2=25 ≤ 100)

    // checked_ilog2 — non-panicking version
    println!("checked_ilog2(0): {:?}", 0u32.checked_ilog2()); // None
    println!("checked_ilog2(8): {:?}", 8u32.checked_ilog2()); // Some(3)
}

// =============================================================================
// 5. Bit Manipulation
// =============================================================================
fn bit_manipulation() {
    println!("\n===== Bit Manipulation =====");

    // Set a bit (force to 1)
    fn set_bit(n: u32, pos: u32) -> u32 { n | (1 << pos) }

    // Clear a bit (force to 0)
    fn clear_bit(n: u32, pos: u32) -> u32 { n & !(1 << pos) }

    // Toggle a bit (flip)
    fn toggle_bit(n: u32, pos: u32) -> u32 { n ^ (1 << pos) }

    // Write a specific bit value (0 or 1)
    fn write_bit(n: u32, pos: u32, val: u32) -> u32 {
        (n & !(1 << pos)) | ((val & 1) << pos)
    }

    // Write a bitfield: set bits [lo..=hi] to value
    fn set_bits(n: u32, lo: u32, hi: u32, val: u32) -> u32 {
        let mask = ((1u32 << (hi - lo + 1)) - 1) << lo;
        (n & !mask) | ((val << lo) & mask)
    }

    let n: u32 = 0b_0000_1010;
    println!("original:     {:08b}", n);                   // 00001010
    println!("set bit 0:    {:08b}", set_bit(n, 0));       // 00001011
    println!("clear bit 1:  {:08b}", clear_bit(n, 1));     // 00001000
    println!("toggle bit 3: {:08b}", toggle_bit(n, 3));    // 00000010
    println!("write bit 4=1:{:08b}", write_bit(n, 4, 1));  // 00011010
    println!("bits[4..=6]=5:{:08b}", set_bits(n, 4, 6, 5)); // 01011010

    // Isolate lowest set bit  (n & -n)
    fn lowest_set_bit(n: i32) -> i32 { n & (-n) }
    println!("lowest set bit of 0b1100: {:04b}", lowest_set_bit(0b1100)); // 0100

    // Clear lowest set bit  (n & (n-1))
    fn clear_lowest_set_bit(n: u32) -> u32 { n & (n - 1) }
    println!("clear lowest of 0b1100:  {:04b}", clear_lowest_set_bit(0b1100)); // 1000

    // Isolate highest set bit
    fn highest_set_bit(n: u32) -> u32 {
        if n == 0 { return 0; }
        1 << (u32::BITS - 1 - n.leading_zeros())
    }
    println!("highest set bit of 100: {}", highest_set_bit(100)); // 64

    // next_power_of_two — round up to nearest power of 2
    let x: u32 = 100;
    println!("next_power_of_two(100): {}", x.next_power_of_two()); // 128
    println!("next_power_of_two(128): {}", 128u32.next_power_of_two()); // 128

    // checked_next_power_of_two — returns None on overflow
    let r = x.checked_next_power_of_two(); // Some(128)
    let r = u32::MAX.checked_next_power_of_two(); // None

    // is_power_of_two — true if exactly one bit is set
    println!("64.is_power_of_two:  {}", 64u32.is_power_of_two());  // true
    println!("100.is_power_of_two: {}", 100u32.is_power_of_two()); // false

    // reverse_bits — mirror all bits
    let n: u8 = 0b_0000_1011;
    println!("reverse_bits: {:08b}", n.reverse_bits()); // 11010000
}

// =============================================================================
// 6. Bit / Byte Conversion
// =============================================================================
fn bit_conversion() {
    println!("\n===== Bit/Byte Conversion =====");

    // to_be_bytes / to_le_bytes / to_ne_bytes
    // Convert integer to raw bytes in big-endian / little-endian / native order
    let n: u32 = 0x01234567;
    let be = n.to_be_bytes();  // [0x01, 0x23, 0x45, 0x67]  most significant first
    let le = n.to_le_bytes();  // [0x67, 0x45, 0x23, 0x01]  least significant first
    let ne = n.to_ne_bytes();  // platform dependent
    println!("BE bytes: {:?}", be);
    println!("LE bytes: {:?}", le);

    // from_be_bytes / from_le_bytes / from_ne_bytes
    // Reconstruct integer from raw bytes
    let back_be = u32::from_be_bytes([0x01, 0x23, 0x45, 0x67]);
    let back_le = u32::from_le_bytes([0x67, 0x45, 0x23, 0x01]);
    assert_eq!(back_be, n);
    assert_eq!(back_le, n);

    // to_be / to_le / to_ne — convert integer value (not bytes)
    let be_val = n.to_be();   // byte-swap if on little-endian machine
    let le_val = n.to_le();   // byte-swap if on big-endian machine
    let from_be = u32::from_be(be_val); // convert from big-endian representation
    let from_le = u32::from_le(le_val);
    assert_eq!(from_be, n);
    assert_eq!(from_le, n);

    // swap_bytes — reverse byte order
    let n: u32 = 0x12345678;
    println!("swap_bytes: {:#010x}", n.swap_bytes()); // 0x78563412

    // from_str_radix — parse integer from string in any base
    let hex  = u32::from_str_radix("FF",   16).unwrap();  // 255
    let bin  = u32::from_str_radix("1010", 2).unwrap();   // 10
    let oct  = u32::from_str_radix("17",   8).unwrap();   // 15
    println!("hex FF = {hex}, bin 1010 = {bin}, oct 17 = {oct}");

    // Bitcast: reinterpret bits as different type (no conversion)
    // f32::to_bits / f32::from_bits
    let f: f32 = 1.0_f32;
    let bits: u32 = f.to_bits();              // 0x3F800000
    println!("1.0f32 bits: {bits:#010x}");
    let back: f32 = f32::from_bits(bits);     // 1.0
    assert_eq!(back, f);

    // f64 version
    let bits: u64 = 1.0_f64.to_bits();
    let back: f64 = f64::from_bits(bits);

    // Transmute alternative: use from_bits/to_bits for f32↔u32, f64↔u64
    // For arbitrary reinterpretation use std::mem::transmute (unsafe)
    let x: i32 = -1;
    let y: u32 = unsafe { std::mem::transmute(x) }; // 0xFFFFFFFF
    println!("transmute -1i32 → u32: {y:#010x}");

    // cast_signed / cast_unsigned (Rust 1.87+)
    // Safe bitcast between signed and unsigned without transmute
    // let y: u32 = (-1i32).cast_unsigned();  // 0xFFFFFFFF
    // let x: i32 = u32::MAX.cast_signed();   // -1
}

// =============================================================================
// 7. Byte Order (Endianness)
// =============================================================================
fn byte_order() {
    println!("\n===== Byte Order =====");

    // Check platform endianness at compile time
    #[cfg(target_endian = "little")]
    println!("This platform is little-endian (x86, ARM, M1)");
    #[cfg(target_endian = "big")]
    println!("This platform is big-endian (SPARC, some MIPS)");

    // Network byte order is big-endian — always use to_be/from_be for network data
    let port: u16 = 8080;
    let network_port = port.to_be();           // ensures big-endian for network
    let host_port = u16::from_be(network_port); // convert back

    // Practical: serialise a u32 to bytes for a binary protocol
    fn write_u32_le(buf: &mut [u8], offset: usize, val: u32) {
        buf[offset..offset+4].copy_from_slice(&val.to_le_bytes());
    }
    fn read_u32_le(buf: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(buf[offset..offset+4].try_into().unwrap())
    }
    let mut buf = [0u8; 8];
    write_u32_le(&mut buf, 0, 0xDEADBEEF);
    let val = read_u32_le(&buf, 0);
    assert_eq!(val, 0xDEADBEEF);
    println!("round-trip: {val:#010x}");
}

// =============================================================================
// 8. Overflow-Aware Arithmetic
// =============================================================================
// Regular +, -, * PANIC in debug on overflow, WRAP in release.
// Use these explicit variants for full control.
// =============================================================================
fn overflow_aware() {
    println!("\n===== Overflow-Aware Arithmetic =====");

    let a: u8 = 250;
    let b: u8 = 10;

    // checked_* — returns None on overflow
    println!("checked_add: {:?}", a.checked_add(b));   // None (260 > 255)
    println!("checked_sub: {:?}", a.checked_sub(b));   // Some(240)
    println!("checked_mul: {:?}", a.checked_mul(b));   // None
    println!("checked_div: {:?}", a.checked_div(2));   // Some(125)
    println!("checked_rem: {:?}", a.checked_rem(3));   // Some(1)
    println!("checked_pow: {:?}", a.checked_pow(2));   // None
    println!("checked_neg: {:?}", 5i8.checked_neg());  // Some(-5)
    println!("checked_abs: {:?}", (-5i8).checked_abs()); // Some(5)
    println!("checked_shl: {:?}", a.checked_shl(1));   // Some(244)
    println!("checked_shr: {:?}", a.checked_shr(1));   // Some(125)

    // saturating_* — clamps to MIN or MAX instead of wrapping/panicking
    println!("saturating_add: {}", a.saturating_add(b)); // 255 (clamped)
    println!("saturating_sub: {}", 5u8.saturating_sub(10)); // 0 (clamped)
    println!("saturating_mul: {}", a.saturating_mul(b)); // 255
    println!("saturating_pow: {}", a.saturating_pow(2)); // 255
    println!("saturating_neg: {}", (-128i8).saturating_neg()); // 127

    // wrapping_* — always wraps (modular arithmetic), no panic
    println!("wrapping_add: {}", a.wrapping_add(b));   // 4  (260 % 256)
    println!("wrapping_sub: {}", 0u8.wrapping_sub(1)); // 255
    println!("wrapping_mul: {}", a.wrapping_mul(b));   // 196
    println!("wrapping_neg: {}", 1u8.wrapping_neg());  // 255
    println!("wrapping_pow: {}", a.wrapping_pow(2));   // 196

    // overflowing_* — (result, did_overflow: bool)
    println!("overflowing_add: {:?}", a.overflowing_add(b)); // (4, true)
    println!("overflowing_sub: {:?}", a.overflowing_sub(b)); // (240, false)
    println!("overflowing_mul: {:?}", a.overflowing_mul(b)); // (196, true)
    println!("overflowing_pow: {:?}", a.overflowing_pow(2)); // (196, true)
    println!("overflowing_neg: {:?}", 1i8.overflowing_neg()); // (-1, false)

    // Widening multiply — get full result without overflow
    // u32::widening_mul → u32 (Rust 1.x, nightly)
    // Stable workaround: widen to u64 first
    let (hi, lo) = {
        let result = a as u64 * b as u64;
        ((result >> 8) as u8, result as u8)
    };
    println!("250 * 10 full: hi={hi} lo={lo}");
}

// =============================================================================
// 9. Classic Bit Tricks
// =============================================================================
fn bit_tricks() {
    println!("\n===== Classic Bit Tricks =====");

    // ---- Power of two checks and rounding ----

    // Is n a power of two? (n > 0 and only one bit set)
    let is_pow2 = |n: u32| n != 0 && (n & (n - 1)) == 0;
    println!("64 pow2: {} | 100 pow2: {}", is_pow2(64), is_pow2(100));

    // Round UP to next power of two
    let round_up_pow2 = |n: u32| n.next_power_of_two();
    println!("next_pow2(100) = {}", round_up_pow2(100)); // 128

    // Round DOWN to previous power of two
    let round_down_pow2 = |n: u32| {
        if n == 0 { return 0u32; }
        1 << (u32::BITS - 1 - n.leading_zeros())
    };
    println!("prev_pow2(100) = {}", round_down_pow2(100)); // 64

    // ---- Swap without temporary ----
    let mut a: i32 = 5;
    let mut b: i32 = 9;
    a ^= b; b ^= a; a ^= b;
    println!("swapped: a={a} b={b}"); // a=9 b=5
    // Note: std::mem::swap is preferred — this is just a classic trick

    // ---- Absolute value without branching (signed integers) ----
    let abs_trick = |n: i32| -> i32 {
        let mask = n >> 31;            // all 0s if positive, all 1s if negative
        (n + mask) ^ mask
    };
    println!("abs(-42) = {}", abs_trick(-42)); // 42
    println!("abs(42)  = {}", abs_trick(42));  // 42

    // ---- Min / Max without branching ----
    let bit_min = |a: i32, b: i32| -> i32 {
        b + ((a - b) & ((a - b) >> 31))
    };
    let bit_max = |a: i32, b: i32| -> i32 {
        a - ((a - b) & ((a - b) >> 31))
    };
    println!("min(3,7) = {} | max(3,7) = {}", bit_min(3, 7), bit_max(3, 7));

    // ---- Sign detection ----
    let sign = |n: i32| -> i32 { (n >> 31) | ((-n) >> 31 & 1) };
    // returns -1, 0, or 1
    println!("sign(-5)={} sign(0)={} sign(5)={}", sign(-5), sign(0), sign(5));

    // ---- Toggle between two values ----
    let toggle = |n: i32, a: i32, b: i32| a ^ b ^ n;
    let mut val = 3;
    val = toggle(val, 3, 7); println!("toggled: {val}"); // 7
    val = toggle(val, 3, 7); println!("toggled: {val}"); // 3

    // ---- Modulo by power of two (fast) ----
    let fast_mod = |n: u32, m: u32| -> u32 {
        // Only works when m is a power of two!
        n & (m - 1)
    };
    println!("100 % 32 = {} (fast: {})", 100 % 32, fast_mod(100, 32)); // 4

    // ---- Count trailing zeros = position of lowest set bit ----
    let n: u32 = 0b_0101_1000;
    println!("lowest set bit position: {}", n.trailing_zeros()); // 3

    // ---- Interleave bits (Morton code / Z-order curve) ----
    // Spread bits of x and y into alternating positions
    fn part1_by1(mut x: u16) -> u32 {
        let mut x = x as u32;
        x = (x | (x << 8)) & 0x00FF00FF;
        x = (x | (x << 4)) & 0x0F0F0F0F;
        x = (x | (x << 2)) & 0x33333333;
        x = (x | (x << 1)) & 0x55555555;
        x
    }
    fn morton_encode(x: u16, y: u16) -> u32 {
        part1_by1(x) | (part1_by1(y) << 1)
    }
    println!("morton(3,5) = {:#034b}", morton_encode(3, 5));

    // ---- Gray code: adjacent values differ by exactly one bit ----
    let to_gray   = |n: u32| n ^ (n >> 1);
    let from_gray = |mut g: u32| -> u32 {
        let mut n = g;
        g >>= 1;
        while g != 0 { n ^= g; g >>= 1; }
        n
    };
    println!("to_gray(6)   = {:04b}", to_gray(6));         // 0101
    println!("from_gray(5) = {}", from_gray(to_gray(6)));  // 6
}

// =============================================================================
// 10. Practical Examples
// =============================================================================
fn practical_examples() {
    println!("\n===== Practical Examples =====");

    // ---- Example 1: Bitflags / Permission system ----
    struct Permissions(u8);
    impl Permissions {
        const READ:    u8 = 1 << 0;  // 0b001
        const WRITE:   u8 = 1 << 1;  // 0b010
        const EXECUTE: u8 = 1 << 2;  // 0b100

        fn new() -> Self { Permissions(0) }
        fn grant(&mut self, flag: u8) { self.0 |= flag; }
        fn revoke(&mut self, flag: u8) { self.0 &= !flag; }
        fn has(&self, flag: u8) -> bool { self.0 & flag == flag }
        fn has_any(&self, flags: u8) -> bool { self.0 & flags != 0 }
    }

    let mut p = Permissions::new();
    p.grant(Permissions::READ | Permissions::WRITE);
    println!("can read:    {}", p.has(Permissions::READ));    // true
    println!("can execute: {}", p.has(Permissions::EXECUTE)); // false
    p.revoke(Permissions::WRITE);
    println!("can write:   {}", p.has(Permissions::WRITE));   // false

    // ---- Example 2: Pack two u16s into one u32 ----
    fn pack(hi: u16, lo: u16) -> u32 {
        ((hi as u32) << 16) | (lo as u32)
    }
    fn unpack(n: u32) -> (u16, u16) {
        ((n >> 16) as u16, n as u16)
    }
    let packed = pack(0xABCD, 0x1234);
    let (hi, lo) = unpack(packed);
    println!("packed: {packed:#010x} → hi={hi:#06x} lo={lo:#06x}");

    // ---- Example 3: Bloom filter (probabilistic set membership) ----
    struct BloomFilter {
        bits: u64,
    }
    impl BloomFilter {
        fn new() -> Self { BloomFilter { bits: 0 } }

        fn hash1(s: &str) -> u64 { s.len() as u64 % 64 }
        fn hash2(s: &str) -> u64 { (s.bytes().sum::<u8>() as u64) % 64 }

        fn insert(&mut self, s: &str) {
            self.bits |= 1 << Self::hash1(s);
            self.bits |= 1 << Self::hash2(s);
        }

        // Returns false → definitely not in set
        // Returns true → probably in set (may have false positives)
        fn probably_contains(&self, s: &str) -> bool {
            let b1 = 1 << Self::hash1(s);
            let b2 = 1 << Self::hash2(s);
            self.bits & b1 != 0 && self.bits & b2 != 0
        }
    }

    let mut bf = BloomFilter::new();
    bf.insert("hello");
    bf.insert("world");
    println!("bloom 'hello': {}", bf.probably_contains("hello")); // true
    println!("bloom 'xyz':   {}", bf.probably_contains("xyz"));   // false (likely)

    // ---- Example 4: Count set bits in a range efficiently ----
    fn count_bits_in_range(lo: u32, hi: u32) -> u32 {
        // Count 1-bits in all integers from lo to hi inclusive
        (lo..=hi).map(|n: u32| n.count_ones()).sum()
    }
    println!("1-bits in 0..=7: {}", count_bits_in_range(0, 7)); // 12

    // ---- Example 5: XOR-based duplicate detection ----
    // XOR of all values where every number appears twice except one
    // cancels out, leaving the unique element
    fn find_unique(nums: &[i32]) -> i32 {
        nums.iter().fold(0, |acc, &x| acc ^ x)
    }
    let nums = [4, 1, 2, 1, 2, 3, 3];           // 4 appears once
    println!("unique element: {}", find_unique(&nums)); // 4

    // ---- Example 6: IP address packing ----
    fn ipv4_to_u32(a: u8, b: u8, c: u8, d: u8) -> u32 {
        (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | d as u32
    }
    fn u32_to_ipv4(ip: u32) -> (u8, u8, u8, u8) {
        ((ip >> 24) as u8, (ip >> 16) as u8, (ip >> 8) as u8, ip as u8)
    }
    let ip = ipv4_to_u32(192, 168, 1, 1);
    println!("ip: {ip:#010x}");
    let (a, b, c, d) = u32_to_ipv4(ip);
    println!("decoded: {a}.{b}.{c}.{d}");        // 192.168.1.1
}

// =============================================================================
// QUICK REFERENCE CARD
// =============================================================================
//
//  OPERATORS          DESCRIPTION
//  &                  AND
//  |                  OR
//  ^                  XOR
//  !                  NOT (bitwise complement)
//  << / >>            left / right shift (logical for u*, arithmetic for i*)
//  rotate_left(n)     rotate bits left by n (wraps around)
//  rotate_right(n)    rotate bits right by n (wraps around)
//  reverse_bits()     mirror all bits
//  swap_bytes()       reverse byte order
//
//  COUNTING           DESCRIPTION
//  count_ones()       population count / popcount / Hamming weight
//  count_zeros()      number of 0-bits
//  leading_zeros()    zeros before first 1 from MSB
//  leading_ones()     ones before first 0 from MSB
//  trailing_zeros()   zeros after last 1 from LSB (= lowest set bit index)
//  trailing_ones()    ones after last 0 from LSB
//
//  INSPECTION         DESCRIPTION
//  is_power_of_two()  true if exactly one bit is set
//  next_power_of_two()  round up to nearest power of 2
//  ilog2()            floor(log base 2) — panics on 0
//  checked_ilog2()    non-panicking log2
//
//  OVERFLOW VARIANTS  DESCRIPTION
//  checked_*(…)       returns Option — None on overflow
//  saturating_*(…)    clamps to MIN/MAX — never overflows
//  wrapping_*(…)      modular / two's complement — never panics
//  overflowing_*(…)   returns (result, did_overflow: bool)
//
//  CONVERSION         DESCRIPTION
//  to_be_bytes()      integer → [u8; N] big-endian
//  to_le_bytes()      integer → [u8; N] little-endian
//  from_be_bytes([])  [u8; N] → integer big-endian
//  from_le_bytes([])  [u8; N] → integer little-endian
//  from_str_radix(s, base) parse string in any base (2..=36)
//  f32::to_bits()     f32 → u32 bitcast (no conversion)
//  f32::from_bits(u)  u32 → f32 bitcast
//
//  BIT MANIPULATION PATTERNS
//  Set bit k:         n |  (1 << k)
//  Clear bit k:       n & !(1 << k)
//  Toggle bit k:      n ^  (1 << k)
//  Test bit k:        (n >> k) & 1  != 0
//  Lowest set bit:    n & n.wrapping_neg()   (or n & -n for signed)
//  Clear lowest bit:  n & (n - 1)
//  Power of 2 check:  n != 0 && n & (n-1) == 0
//  Fast mod pow2:     n & (m - 1)            (only when m is power of 2)
//  XOR swap:          a^=b; b^=a; a^=b
//
// =============================================================================