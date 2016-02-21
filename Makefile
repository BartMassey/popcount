# Copyright (c) 2014 Bart Massey
# ALL RIGHTS RESERVED
# [This program is licensed under the GPL version 3 or later.]
# Please see the file COPYING in the source
# distribution of this software for license terms.

CC = gcc -O4
# CC = clang -O3

# Replace the machine with yours.  Don't use X86_POPCNT
# unless you are on a Nehalem or later Intel processor. Some
# checking will be done on Intel/AMD parts, but things
# won't even compile (assemble) on non-x86.
CFLAGS = -Wall -march=core-avx2 -DX86_POPCNT

popcount: popcount.o

popcount.o:  popcount_table_8.c popcount_table_16.c

popcount_table_8.c: make_table.5c
	nickle make_table.5c 8 >$*.c

popcount_table_16.c: make_table.5c
	nickle make_table.5c 16 >$*.c

clean:
	-rm -f popcount.o popcount

distclean: clean
	-rm -f popcount_table_8.c popcount_table_16.c
