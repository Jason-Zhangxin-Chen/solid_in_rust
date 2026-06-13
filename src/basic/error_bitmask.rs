/// A high performance error handling base on bit mask in Rust for some ultra low latency scenario
/// like trade system or gaming system.

pub struct ErrorCode(u128);

pub const MAX_PRICE: f64 = 99999999.999f64;

impl ErrorCode {
    pub const OK: Self = ErrorCode(0);
    pub const PARSE_ERROR: Self = ErrorCode(1 << 0);
    pub const INVALID_PARA: Self = ErrorCode(1 << 1);
    pub const RATE_LIMIT: Self = ErrorCode(1 << 2);
    pub const INSUFFICIENT_FUND: Self = ErrorCode(1 << 3);
    pub const INVALID_PRICE: Self = ErrorCode(1 << 4);
    pub const EXCEEDS_LIMIT: Self = ErrorCode(1 << 5);
    // todo: add the error list here.

    #[inline(always)]
    pub fn or(&self, other: ErrorCode) -> ErrorCode {
        ErrorCode(self.0 | other.0)
    }

    #[inline(always)]
    pub fn is_ok(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn has_error(&self, other: ErrorCode) -> bool {
        (self.0 & other.0) != 0
    }
}

pub struct Checker {
    err: ErrorCode,
}

impl Checker {
    #[inline(always)]
    pub fn new() -> Self { Checker { err: ErrorCode::OK } }

    #[inline(always)]
    pub fn check(&mut self, condition: bool, flag: ErrorCode) {
        if condition {
            self.err.or(flag);
        }
    }

    #[inline(always)]
    pub fn is_ok(&self) -> bool { self.err.is_ok() }

    #[inline(always)]
    pub fn finish(self) -> ErrorCode {
        self.err
    }
}


fn validate_price(price: f64) -> (f64, ErrorCode) {
    let mut checker = Checker::new();
    checker.check(price <= 0.0, ErrorCode::INVALID_PRICE);
    if !checker.is_ok() {
        return (price, checker.err)
    }

    checker.check(price > MAX_PRICE, ErrorCode::EXCEEDS_LIMIT);
    if !checker.is_ok() {
        return (price, checker.err)
    }

    (price, ErrorCode::OK)
}
