#![no_std]
#![forbid(unsafe_code)]

pub mod google {
    pub mod protobuf {
        pub mod descriptor;
        pub use descriptor::*;
    }
}
