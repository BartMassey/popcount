# Copyright (c) 2014 Bart Massey
# [This program is licensed under the "MIT License"]
# Please see the file LICENSE in the source
# distribution of this software for license terms.

GCC = gcc -O4
CLANG = clang -O3

GCCFLAGS = -Wall -march=native
CLANGFLAGS = -Wall -march=native

TARGETS = popcount_gcc popcount_clang popcount_rs

all: $(TARGETS)

popcount_gcc: popcount.c
	$(GCC) $(GCCFLAGS) -o popcount_gcc popcount.c

popcount_clang: popcount.c
	$(CLANG) $(CLANGFLAGS) -o popcount_clang popcount.c

target/release/popcount_rs: popcount.rs
	RUSTFLAGS='-C target-cpu=native' cargo build --release

popcount_rs: target/release/popcount_rs
	cp target/release/popcount_rs .

clean:
	-rm -f $(TARGETS)
	cargo clean
