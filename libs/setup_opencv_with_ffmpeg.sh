# !/bin/bash

COMPILE_THREADS=3

# FFmpeg installation
echo "Installing FFMPEG ..."
wget https://ffmpeg.org/releases/ffmpeg-3.4.8.tar.xz
tar -xf ffmpeg-3.4.8.tar.xz 
rm ffmpeg-3.4.8.tar.xz
cd ffmpeg-3.4.8
mkdir build
PREFIX="$PWD/build"
PATH="$PREFIX/bin:$PATH" 
PKG_CONFIG_PATH="$PREFIX/pkgconfig"
./configure --prefix=$PREFIX --enable-nonfree --enable-pic --enable-shared
make -j$COMPILE_THREADS
make install
cd ..
FFMPEG_HOME=$PWD/ffmpeg-3.4.8/build
export PATH=${FFMPEG_HOME}/bin:$PATH
export LD_LIBRARY_PATH=${FFMPEG_HOME}/lib:${FFMPEG_HOME}/include:$LD_LIBRARY_PATH
export PKG_CONFIG_PATH=${FFMPEG_HOME}/lib/pkgconfig:$PKG_CONFIG_PATH

# OpenCV installation
echo "Installing OpenCV ..."
wget https://github.com/opencv/opencv/archive/4.5.0.zip
unzip 4.5.0.zip
rm 4.5.0.zip
cd opencv-4.5.0/
git clone https://github.com/opencv/opencv_contrib.git opencv_contrib
mkdir build
cd build
cmake -DOPENCV_GENERATE_PKGCONFIG=YES -DOPENCV_EXTRA_MODULES_PATH=../opencv_contrib/modules/face/ -DBUILD_PNG=ON -DBUILD_EXAMPLES=OFF -DWITH_FFMPEG=ON -DOPENCV_FFMPEG_SKIP_BUILD_CHECK=ON -DCMAKE_INSTALL_PREFIX=../ ..
make -j$COMPILE_THREADS
make install
cd ../../

echo "Done. Don't forget to run: $ source config_vars_opencv.sh"
