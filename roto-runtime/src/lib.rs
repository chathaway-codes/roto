#![no_std]
#![forbid(unsafe_code)]

/// Compile-time and runtime verification marker confirming `roto-runtime` is linked.
#[inline]
pub const fn runtime_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_is_ready() {
        assert!(runtime_ready());
    }
}
