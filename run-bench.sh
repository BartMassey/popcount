#!/bin/sh
# Copyright © 2018 Bart Massey
# [This program is licensed under the "MIT License"]
# Please see the file LICENSE in the source
# distribution of this software for license terms.

for i in popcount_gcc popcount_clang popcount_rs
do
    echo ""
    echo $i
    ./$i 1000000
done
