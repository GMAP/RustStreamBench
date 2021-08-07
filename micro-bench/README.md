# Micro-bench #

Synthetic application in RUST.

This application computes a fractal in the complex plane named Mandelbrot Set.

# Inputs

This application does not require an input source file. 

Instead, you can specify the dimensions of the matrix and iteration boundaries.

# List of dependencies

	llvm
	clang 
	libclang-dev
	Other dependencies in Cargo.toml file

# Basic commands for running

	cargo build --release
	./<path_to_binary> <runtime> <matrix dim> <nthreads> <iter1 boundary> <iter2 boundary>

Options for `runtime` are: 
	"sequential", or "rust-ssp", or "pipeliner", or "tokio", or "rayon", or "std-threads" 

	
Command example:

`$ ./target/release/micro-bench sequential 2048 1 3000 2000`
