use std::env;
use std::path::PathBuf;
use std::process::Command;

const PROTOS: &[&str] = &[
    "benchmarks.proto",
    "benchmark_message1_proto2.proto",
    "benchmark_message1_proto3.proto",
    "perftest_data.proto",
];

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_dir = manifest_dir.join("proto");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    for proto in PROTOS {
        println!("cargo:rerun-if-changed={}", proto_dir.join(proto).display());
    }
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("c/upb_bench_shim.c").display()
    );

    let proto_paths: Vec<PathBuf> = PROTOS.iter().map(|p| proto_dir.join(p)).collect();

    // 1. Generate prost code
    prost_build::compile_protos(&proto_paths, &[&proto_dir])
        .expect("failed to compile protos with prost-build");

    // 2. Generate upb C headers and minitables
    let upb_out = out_dir.join("upb_gen");
    std::fs::create_dir_all(&upb_out).unwrap();
    let status = Command::new("protoc")
        .arg(format!("--proto_path={}", proto_dir.display()))
        .arg(format!("--upb_out={}", upb_out.display()))
        .arg(format!("--upb_minitable_out={}", upb_out.display()))
        .args(PROTOS)
        .status()
        .expect("failed to execute protoc for upb");
    assert!(status.success(), "protoc upb codegen failed");

    // 3. Compile upb minitables and C shim
    let mut build = cc::Build::new();
    build
        .include(&upb_out)
        .include("/usr/include")
        .opt_level(3)
        .file(manifest_dir.join("c/upb_bench_shim.c"))
        .file(upb_out.join("benchmarks.upb_minitable.c"))
        .file(upb_out.join("benchmark_message1_proto2.upb_minitable.c"))
        .file(upb_out.join("benchmark_message1_proto3.upb_minitable.c"))
        .file(upb_out.join("perftest_data.upb_minitable.c"));
    build.compile("upb_bench_shim");

    println!("cargo:rustc-link-search=native=/usr/lib");
    println!("cargo:rustc-link-lib=static=upb");
    println!("cargo:rustc-link-lib=dylib=utf8_range");
}
