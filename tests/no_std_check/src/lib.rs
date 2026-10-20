#![no_std]
#![forbid(unsafe_code)]

#[cfg(all(not(test), target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Exercises `roto` and `roto-runtime` in a `#![no_std]`, zero-allocation environment.
pub fn verify_no_std() -> bool {
    roto::runtime::runtime_ready() && roto_runtime::runtime_ready()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_std_harness_smoke_test() {
        assert!(verify_no_std());
    }
}
