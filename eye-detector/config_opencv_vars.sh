#!/bin/bash

FFMPEG_HOME=$PWD/../libs/ffmpeg-3.4.8/build
OPENCV_HOME=$PWD/../libs/opencv-4.5.0

export PATH=${FFMPEG_HOME}/bin:${OPENCV_HOME}/bin:$PATH
export LD_LIBRARY_PATH=${FFMPEG_HOME}/lib:${FFMPEG_HOME}/include:${OPENCV_HOME}/lib:${OPENCV_HOME}/include:$LD_LIBRARY_PATH
export PKG_CONFIG_PATH=${FFMPEG_HOME}/lib/pkgconfig:${OPENCV_HOME}/lib/pkgconfig:$PKG_CONFIG_PATH
