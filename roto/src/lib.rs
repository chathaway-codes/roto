#![no_std]
#![forbid(unsafe_code)]

pub use roto_runtime as runtime;

#[cfg(test)]
mod tests {
    use crate::runtime;

    #[test]
    fn reexports_roto_runtime() {
        assert!(runtime::runtime_ready());
    }
}
