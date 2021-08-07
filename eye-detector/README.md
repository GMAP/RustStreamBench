# Eye Detector #

Eye detector application in RUST.

This application detects faces and eyes in a video.

All computer vision algorithms are provided by OpenCV library version 4.5.

# Inputs

Classifiers may be found in config directory:

	haarcascade_frontalface classifier
	haarcascade_eye classifier
	input video

You may get these inputs using the `get_inputs.sh` shell script.

# List of dependencies

	llvm
	clang 
	libclang-dev
	Other dependencies in Cargo.toml file

# Basic commands for running
	source ./config_opencv_vars.sh
	cargo build --release
	./<path_to_binary> <runtime> <nthreads> <input_video>

Options for `runtime` are: 
	"seq", "rust-spp", "tokio", "std-threads", or "better"
	where "better" is the "*std-threads" equivalent from the paper.
