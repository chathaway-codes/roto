# Built from the grpc_bench root as context (see ../run.sh), which stages a snapshot of
# this repo into rust_roto_bench/roto and the scenario proto into proto/.
FROM rust:1.94

RUN apt-get update && apt-get install -y protobuf-compiler

COPY rust_roto_bench/roto /roto
COPY proto/helloworld/helloworld.proto /roto/grpc-bench/rust_roto_bench/proto/helloworld.proto

WORKDIR /roto
RUN cargo build --release --locked -p cargo-roto -p protoc-gen-roto -p protoc-gen-roto-grpc

WORKDIR /roto/grpc-bench/rust_roto_bench
RUN /roto/target/release/cargo-roto roto && cargo build --release --locked

ENTRYPOINT ["/roto/grpc-bench/rust_roto_bench/target/release/helloworld-server"]
