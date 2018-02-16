# Copyright (c) 2014 Bart Massey
# ALL RIGHTS RESERVED
# [This program is licensed under the GPL version 3 or later.]
# Please see the file COPYING in the source
# distribution of this software for license terms.

GCC = gcc -O4
CLANG = clang -O3

# Replace the machine with yours.  Don't use X86_POPCNT
# unless you are on a Nehalem or later Intel processor. Some
# checking will be done on Intel/AMD parts, but things
# won't even compile (assemble) on non-x86.
CFLAGS = -Wall -march=native -DX86_POPCNT

TARGETS = popcount_gcc popcount_clang popcount_rs

popcount_gcc: popcount.c
	$(GCC) $(CFLAGS) -o popcount_gcc popcount.c

all: $(TARGETS)

popcount_clang: popcount.c
	$(CLANG) $(CFLAGS) -o popcount_clang popcount.c

popcount_rs: popcount.rs
	RUSTFLAGS='-C target-cpu=native' cargo build --release
	cp target/release/popcount_rs .

clean:
	-rm -f $(TARGETS)
	cargo clean
