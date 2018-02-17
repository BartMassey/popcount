#!/bin/sh
# Copyright © 2018 Bart Massey
# [This program is licensed under the GPL version 3 or later.]
# Please see the file COPYING in the source
# distribution of this software for license terms.

for i in popcount_gcc popcount_clang popcount_rs
do
    echo $i
    ./$i 1000000
    echo ""
done
