pub mod roto_gen;
pub use roto_gen::benchmarks;
pub use roto_gen::perftest_data;

pub mod prost_gen {
    pub mod benchmarks {
        include!(concat!(env!("OUT_DIR"), "/benchmarks.rs"));
        pub mod proto2 {
            include!(concat!(env!("OUT_DIR"), "/benchmarks.proto2.rs"));
        }
        pub mod proto3 {
            include!(concat!(env!("OUT_DIR"), "/benchmarks.proto3.rs"));
        }
    }
    pub mod perftest_data {
        include!(concat!(env!("OUT_DIR"), "/perftest_data.rs"));
    }
}

pub mod upb_ffi {
    use core::ffi::c_void;

    #[repr(C)]
    pub struct UpbArena {
        _priv: [u8; 0],
    }

    #[repr(C)]
    pub struct UpbGm1Proto2 {
        _priv: [u8; 0],
    }

    #[repr(C)]
    pub struct UpbGm1Proto3 {
        _priv: [u8; 0],
    }

    #[repr(C)]
    pub struct UpbPerftestData {
        _priv: [u8; 0],
    }

    #[repr(C)]
    pub struct UpbTestOptionalMessages {
        _priv: [u8; 0],
    }

    #[repr(C)]
    pub struct UpbTestRepeatedMessages {
        _priv: [u8; 0],
    }

    extern "C" {
        pub fn upb_bench_arena_new() -> *mut UpbArena;
        pub fn upb_bench_arena_init(mem: *mut c_void, n: usize) -> *mut UpbArena;
        pub fn upb_bench_arena_free(arena: *mut UpbArena);

        pub fn upb_gm1_proto2_parse(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> *mut UpbGm1Proto2;
        pub fn upb_gm1_proto2_partial_read(msg: *const UpbGm1Proto2) -> u64;
        pub fn upb_gm1_proto2_traverse_all(msg: *const UpbGm1Proto2) -> u64;
        pub fn upb_gm1_proto2_serialize(
            msg: *const UpbGm1Proto2,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;
        pub fn upb_gm1_proto2_build_and_serialize(
            src: *const UpbGm1Proto2,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_gm1_proto3_parse(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> *mut UpbGm1Proto3;
        pub fn upb_gm1_proto3_partial_read(msg: *const UpbGm1Proto3) -> u64;
        pub fn upb_gm1_proto3_traverse_all(msg: *const UpbGm1Proto3) -> u64;
        pub fn upb_gm1_proto3_serialize(
            msg: *const UpbGm1Proto3,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;
        pub fn upb_gm1_proto3_build_and_serialize(
            src: *const UpbGm1Proto3,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_perftest_parse(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> *mut UpbPerftestData;
        pub fn upb_perftest_partial_read(msg: *const UpbPerftestData) -> u64;
        pub fn upb_perftest_traverse_all(msg: *const UpbPerftestData) -> u64;
        pub fn upb_perftest_serialize(
            msg: *const UpbPerftestData,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_test1_parse_and_read(buf: *const u8, len: usize, arena: *mut UpbArena) -> u64;
        pub fn upb_test1_build_and_serialize(
            val: i32,
            has_val: bool,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_test_repeated_bool_parse_and_read(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> u64;
        pub fn upb_test_repeated_bool_build_and_serialize(
            vals: *const bool,
            n: usize,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_test_packed_int32_parse_and_read(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> u64;
        pub fn upb_test_packed_int32_build_and_serialize(
            vals: *const i32,
            n: usize,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_test_strings_parse_and_read(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> u64;
        pub fn upb_test_strings_build_and_serialize(
            s1: *const u8,
            l1: usize,
            has1: bool,
            s2: *const u8,
            l2: usize,
            has2: bool,
            s3: *const u8,
            l3: usize,
            has3: bool,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_test_bytes_parse_and_read(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> u64;
        pub fn upb_test_bytes_build_and_serialize(
            b1: *const u8,
            l1: usize,
            has1: bool,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_test_optional_messages_parse_and_read(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> u64;
        pub fn upb_test_optional_messages_parse(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> *mut UpbTestOptionalMessages;
        pub fn upb_test_optional_messages_build_and_serialize(
            src: *const UpbTestOptionalMessages,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;

        pub fn upb_test_repeated_messages_parse_and_read(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> u64;
        pub fn upb_test_repeated_messages_parse(
            buf: *const u8,
            len: usize,
            arena: *mut UpbArena,
        ) -> *mut UpbTestRepeatedMessages;
        pub fn upb_test_repeated_messages_build_and_serialize(
            src: *const UpbTestRepeatedMessages,
            arena: *mut UpbArena,
            out_len: *mut usize,
        ) -> *const u8;
    }
}

pub mod datasets {
    pub const GOOGLE_MESSAGE1_PROTO2: &[u8] =
        include_bytes!("../datasets/dataset.google_message1_proto2.pb");
    pub const GOOGLE_MESSAGE1_PROTO3: &[u8] =
        include_bytes!("../datasets/dataset.google_message1_proto3.pb");
    pub const PERFTEST_DATA: &[u8] = include_bytes!("../datasets/perftest_data.pbbin");
}
