# Bzip2 #

Bzip2 application in RUST.

This application compresses and decompresses files using bzip2.

Bzip2 core routines contain binddings to libbz2. This library is provided by Bzip2 for Rust (https://crates.io/crates/bzip2).

# Inputs

Input is a directory of files that can be compressed and decompressed using Bzip2.

You may get the inputs using the `get_inputs.sh` shell script.

We provide workloads with different behaviours. You will find more information about the workload characteristics in our paper.


# List of dependencies

	llvm
	clang 
	libclang-dev
	bzip2-sys
	Other dependencies in Cargo.toml file

# Basic commands for running
	cargo build --release
	./<path_to_binary> <runtime> <nthreads> <compress/decompress> <input_file>

Options for `runtime` are: 
	"sequential", or "rust-ssp", or "pipeliner", or "tokio", or "rayon", or "std-threads"

Alternative `runtime` options are (see paper):
	"sequential-io", or "rust-ssp-io", or "tokio-io", or "std-threads-io"

Command example:

`$ ./target/release/bzip2 rust-ssp 4 compress iso_file.iso`
