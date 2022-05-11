#!/bin/bash

mkdir input_big
mkdir input_small
mkdir input_mixed

#Big input
for((a=0;a<1000;a++)) do
	cp inputs/big.jpg input_big/big$a.jpg
done;

#Small input
for((a=0;a<1000;a++)) do
	cp inputs/small.jpg input_small/small$a.jpg
done;

#Mixed input
for((a=0;a<500;a++)) do
	cp inputs/big.jpg input_mixed/big$a.jpg
	cp inputs/small.jpg input_mixed/small$a.jpg
done;