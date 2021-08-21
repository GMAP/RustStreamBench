# Rust Stream Bench

**The Rust Stream Bench** is a benchmark suite for evaluating stream parallelism on parallel APIs available in **Rust** language. Our [paper](https://doi.org/10.1016/j.cola.2021.101054) contains abundant information about the benchmark applications and workload characteristics. In the paper, we also discussed the methodology used and the outcome of performance evaluation. **You can use our paper as a guide to assess performance on different Rust features using this benchmark suite.**

This project was conducted in the Parallel Applications Modelling Group (GMAP) at PUCRS - Brazil.

## How to cite this work
  
[[DOI]](https://doi.org/10.1016/j.cola.2021.101054) R. Pieper, J. Löff, R.B. Hoffmann, D. Griebler, L. G. Fernandes. **High-level and efficient structuredstream parallelism for rust on multi-cores**, *Journal of Computer Languages (COLA)* (2021)

*You can also contribute with this project, writing issues and pull requests.*

# The Four Benchmark Applications

- **Micro-bench** - Is a mathematical synthetic application that was designed using a fractal in the complex plane, namely the Mandelbrot set. Contains unbalanced workload that requires efficient schedulers.

- **Image Processing** - Is characterized by a stream of images flowing through 5 different filters: Saturation, Emboss, GammaCorrection, Sharpen, and Grayscale. Some filters may take longer than others to complete and all of them are stateless.

- **Bzip2** - Is an important open-source tool for loss-less data compression/decompression. Compression is computationally costly whereas the decompression is shipper. Also, performance heavily depends on the characteristics of the input file. 

- **Eye Detector** - Is a video processing application that detects the eyes in the faces within an input video. Performance heavily depends on the characteristics of the input video. More faces in the video require more computational power. 

_Tip: Find more information in our [paper](https://doi.org/10.1016/j.cola.2021.101054)_

## Folders inside the  project

- **bzip2** - This directory contains the *Bzip2* benchmark application with suitable parallel implementations.
- **eye-detector** - This directory contains the *Eye Detector* benchmark application with suitable parallel implementations.
- **image-processing** - This directory contains the *Image Processing* benchmark application with suitable parallel implementations.
- **micro-bench** - This directory contains the *Micro-bench* benchmark application with suitable parallel implementations.
- **libs** - This directory contains our *Rust-SSP* parallel programming API lib and *OpenCV* installation script for the _Eye Detector_ application.


# How to Compile 

Programs can be compiled using default Cargo Rust package manager.
Inside each application directory, you may do so using:
	`$ cargo build --release`

# How to Execute

You can find information about how to compile and execute the applications in a `README.md` inside each folder.

Besides we provide the input workloads we used in our experimental evaluation. You can download them using the `get_inputs.sh` script that is within each application folder. Except Micro-bench, which does not require any input source.

# How to install OpenCV

You can install OpenCV version 4.5 and Rust wrapper following these steps:

`$ cd libs/`

`$ source setup_opencv_with_ffmpeg.sh`

`$ source config_opencv_vars.sh`

Remember to adjust the COMPILE_THREADS variable inside the `setup_opencv_with_ffmpeg.sh` script accordingly.
Whenever you open a new session, you will need to set OpenCV enviromental variables with `config_opencv_vars.sh` script.
