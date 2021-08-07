# Image Processing #

Image Processing application in RUST.

This application simply applies a sequence of filters over a set of images inside a directory.

All filters are provided by Raster.

# Inputs

Input is a directory of images to be filtered.
You may get the inputs using the `get_inputs.sh` shell script.
For testing, you may replicate these inputs using the `create_inputs.sh` shell script, obtained from the previous script.

# List of dependencies

	llvm
	clang 
	libclang-dev
	Other dependencies in Cargo.toml file

# Basic commands for running
	cargo build --release
	./<path_to_binary> <runtime> <nthreads> <images dir>

Options for `runtime` are: 
	"sequential", or "rust-ssp", or "pipeliner", or "tokio", or "rayon", or "std-threads" 

