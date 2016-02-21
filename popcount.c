/*
 * Copyright (c) 2007 Bart Massey
 * ALL RIGHTS RESERVED
 * 
 * [This program is licensed under the GPL version 3 or later.]
 * Please see the file COPYING in the source
 * distribution of this software for license terms.
 */

#include <assert.h>
#include <stdio.h>
#include <inttypes.h>
#include <sys/time.h>
#include <time.h>

/* A block of random values for popcount to
   repeatedly operate on. */
#define BLOCKSIZE 1000
uint32_t randoms[BLOCKSIZE];

/* XXX Because the popcount routine wants to be inlined
   in the loop, we need to expand each popcount routine
   in its own driver. */
#define DRIVER(NAME) \
  uint32_t                                                   \
  drive_##NAME(int n) {                                      \
    int i, j;                                                \
    uint32_t result = 0;                                     \
    for (j = 0; j < n; j++)                                  \
	for (i = 0; i < BLOCKSIZE; i++)                      \
	    result += popcount_##NAME(randoms[i] ^ result);  \
    return result;   \
  }


/* Baseline */
/* 96 ops, 64 stages */
static inline uint32_t
popcount_naive(uint32_t n) {
    uint32_t c = 0;
    while (n) {
	c += n & 1;
	n >>= 1;
    }
    return c;
}
DRIVER(naive)


/* bit-parallelism */
/* 27 ops, 1 long immediate, 20 stages */
static inline uint32_t
popcount_8(uint32_t n) {
    uint32_t m = 0x01010101;
    uint32_t c = n & m;
    int i;
    for (i = 0; i < 7; i++) {
	n >>= 1;
	c += n & m;
    }
    c += c >> 8;
    c += c >> 16;
    return c & 0x3f;
}
DRIVER(8)


/* more bit-parallelism */
/* 23 ops, 1 long immediate, 18 stages */
static inline uint32_t
popcount_6(uint32_t n) {
    uint32_t m = 0x41041041;
    uint32_t c = n & m;
    int i;
    for (i = 0; i < 5; i++) {
	n >>= 1;
	c += n & m;
    }
    c += c >> 6;
    c += c >> 12;
    c += c >> 24;
    return c & 0x3f;
}
DRIVER(6)


/* HAKMEM 169 */
/* 9 ops plus divide, 2 long immediates, 9 stages */
static inline uint32_t
popcount_hakmem(uint32_t mask)
{
    uint32_t y;

    y = (mask >> 1) & 033333333333;
    y = mask - y - ((y >>1) & 033333333333);
    return ((y + (y >> 3)) & 030707070707) % 63;
}
DRIVER(hakmem)


/* Joe Keane, sci.math.num-analysis, 9 July 1995,
   as cited by an addendum to Hacker's Delight.
   http://www.hackersdelight.org/divcMore.pdf */
static inline uint32_t
remu63(uint32_t n) {
    uint32_t t = (((n >> 12) + n) >> 10) + (n << 2);
    t = ((t >> 6) + t + 3) & 0xff;
    return (t - (t >> 6)) >> 2;
}

/* HAKMEM 169 with Keane modulus */
/* 9 + 12 = 21 ops, 2 long immediates, 14 stages */
static inline uint32_t
popcount_keane(uint32_t mask)
{
    uint32_t y;

    y = (mask >> 1) & 033333333333;
    y = mask - y - ((y >>1) & 033333333333);
    return remu63((y + (y >> 3)) & 030707070707);
}
DRIVER(keane)


/* Divide-and-conquer with a ternary stage to reduce masking */
/* 17 ops, 2 long immediates, 12 stages, 14 alu ops, 11 alu stages  */
static inline uint32_t
popcount_3(uint32_t x)
{
    uint32_t m1 = 0x55555555;
    uint32_t m2 = 0xc30c30c3;
    x -= (x >> 1) & m1;
    x = (x & m2) + ((x >> 2) & m2) + ((x >> 4) & m2); 
    x += x >> 6;
    return  (x + (x >> 12) + (x >> 24)) & 0x3f;
}
DRIVER(3)


/* Divide-and-conquer with a quaternary stage to reduce masking
   and provide mostly power-of-two shifts */
/* 18 ops, 2 long immediates, 12 stages, 12 alu ops, 9 alu stages */
static inline uint32_t
popcount_4(uint32_t x)
{
    uint32_t m1 = 0x55555555;
    uint32_t m2 = 0x03030303;
    x -= (x >> 1) & m1;
    x = (x & m2) + ((x >> 2) & m2) + ((x >> 4) & m2) + ((x >> 6) & m2);
    x += x >> 8;
    return  (x + (x >> 16)) & 0x3f;
}
DRIVER(4)


    /* Classic binary divide-and-conquer popcount.
       This is popcount_2() from
       http://en.wikipedia.org/wiki/Hamming_weight */
    /* 15 ops, 3 long immediates, 14 stages, 9 alu ops, 9 alu stages */
    static inline uint32_t
    popcount_2(uint32_t x)
    {
	uint32_t m1 = 0x55555555;
	uint32_t m2 = 0x33333333;
	uint32_t m4 = 0x0f0f0f0f;
	x -= (x >> 1) & m1;
	x = (x & m2) + ((x >> 2) & m2);
	x = (x + (x >> 4)) & m4;
	x += x >>  8;
	return (x + (x >> 16)) & 0x3f;
    }
DRIVER(2)


/* Popcount using multiply.
   This is popcount_3() from
   http://en.wikipedia.org/wiki/Hamming_weight */
/* 11 ops plus 1 multiply, 4 long immediates, 11 stages */
static inline uint32_t
popcount_mult(uint32_t x)
{
    uint32_t m1 = 0x55555555;
    uint32_t m2 = 0x33333333;
    uint32_t m4 = 0x0f0f0f0f;
    uint32_t h01 = 0x01010101;
    x -= (x >> 1) & m1;   /* put count of each 2 bits into those 2 bits */
    x = (x & m2) + ((x >> 2) & m2);   /* put count of each 4 bits in */
    x = (x + (x >> 4)) & m4;  /* put count of each 8 bits in */
    return (x * h01) >> 24;  /* returns left 8 bits of x + (x<<8) + ... */
}
DRIVER(mult)


struct drivers {
    char *name;
    uint32_t (*f)(uint32_t);
    uint32_t (*blockf)(int);
    uint32_t divisor;
};

#include "popcount_table_8.c"

/* Table-driven popcount, with 8-bit tables */
/* 6 ops plus 4 casts and 4 lookups, 0 long immediates, 4 stages */
static inline uint32_t
popcount_tabular_8(uint32_t x)
{
    return popcount_table_8[(uint8_t)x] +
	   popcount_table_8[(uint8_t)(x >> 8)] +
	   popcount_table_8[(uint8_t)(x >> 16)] +
	   popcount_table_8[(uint8_t)(x >> 24)];
}
DRIVER(tabular_8)

#include "popcount_table_16.c"

/* Table-driven popcount, with 16-bit tables */
/* 2 ops plus 2 casts and 2 lookups, 0 long immediates, 4 stages */
static inline uint32_t
popcount_tabular_16(uint32_t x)
{
    return popcount_table_16[(uint16_t)x] +
	   popcount_table_16[(uint16_t)(x >> 16)];
}
DRIVER(tabular_16)


#ifdef X86_POPCNT
static int
has_popcnt_x86() {
    uint32_t eax, ecx;
    eax = 0x01;
    asm("cpuid" : "+a" (eax), "=c" (ecx) : : "ebx", "edx");
    return (ecx >> 23) & 1;
}


/* x86 popcount instruction */
/* 3 cycle latency, 1 cycle throughput */
static inline uint32_t
popcount_x86(uint32_t x)
{
    uint32_t result;
    asm("popcntl %1, %0" : "=r" (result) : "r" (x) : "cc");
    return result;
}
DRIVER(x86)
#endif

struct drivers drivers[] = {
/*  {"popcount_swar", popcount_swar, drive_swar, 1}, */
    {"popcount_naive", popcount_naive, drive_naive, 8},
    {"popcount_8", popcount_8, drive_8, 1},
    {"popcount_6", popcount_6, drive_6, 1},
    {"popcount_hakmem", popcount_hakmem, drive_hakmem, 1},
    {"popcount_keane", popcount_keane, drive_keane, 1},
    {"popcount_3", popcount_3, drive_3, 1},
    {"popcount_4", popcount_4, drive_4, 1},
    {"popcount_2", popcount_2, drive_2, 1},
    {"popcount_mult", popcount_mult, drive_mult, 1},
    {"popcount_tabular_8", popcount_tabular_8, drive_tabular_8, 1},
    {"popcount_tabular_16", popcount_tabular_16, drive_tabular_16, 1},
#ifdef X86_POPCNT
    {"popcount_x86", popcount_x86, drive_x86, 1},
#endif
    {0, 0, 0}
};


void
init_randoms() {
    extern long random();
    int i;
    for (i = 0; i < BLOCKSIZE; i++)
	randoms[i] = random() ^ (random() >> 16) ^ (random() << 16);
}


uint32_t
elapsed_msecs(struct timeval *start,
	      struct timeval *end) {
    int32_t diff = (end->tv_sec - start->tv_sec) * 1000;
    int32_t endu = end->tv_usec / 1000;
    int32_t startu = start->tv_usec / 1000;
    return diff + endu - startu;
}


struct testcases {
    uint32_t input, output;
};

struct testcases testcases[] = {
    {0x00000080, 1},
    {0x000000f0, 4},
    {0x00008000, 1},
    {0x0000f000, 4},
    {0x00800000, 1},
    {0x00f00000, 4},
    {0x80000000, 1},
    {0xf0000000, 4},
    {0xff000000, 8},
    {0x000000ff, 8},
    {0x01fe0000, 8},
    {0xea9031e8, 14},
    {0x2e8eb2b2, 16},
    {0x9b8be5b7, 20},
    {~0, 32},
    {0, 0}
};


void
test_driver(struct drivers *d) {
    struct testcases *t;
    int nt = 1;
    uint32_t last = 1;
    for (t = testcases; last != 0; t++) {
	uint32_t output = (d->f)(t->input);
	if (output != t->output) {
	    printf("%s failed case %d: %x -> %u != %u: abandoning\n",
		   d->name, nt, t->input, output, t->output);
	    d->blockf = 0;
	    return;
	}
	last = t->input;
	nt++;
    }
}


void
run_driver(struct drivers *d, int n) {
    volatile uint32_t result = 0;
    struct timeval start, end;
    uint32_t elapsed;
    uint32_t real_n = n / d->divisor;
    /* preheat */
    result += d->blockf(5000 / d->divisor);
    assert(gettimeofday(&start, 0) != -1);
    result += d->blockf(real_n);
    assert(gettimeofday(&end, 0) != -1);
    elapsed = elapsed_msecs(&start, &end);
    printf("%s: %g iters in %d msecs for %0.2f nsecs/iter\n",
	   d->name, (double)real_n * BLOCKSIZE, elapsed,
	   elapsed * d->divisor * 1.0e6 / BLOCKSIZE / n);
}


void
run_all(int n) {
    struct drivers *d;
    for (d = drivers; d->name; d++)
	test_driver(d);
    for (d = drivers; d->name; d++)
	if (d->blockf)
	    run_driver(d, n);
}


int
main(int argc,
     char **argv) {
    extern long atoi();
    int n = atoi(argv[1]);
#ifdef X86_POPCNT
    assert(has_popcnt_x86());
#endif
    init_randoms();
    run_all(n);
    return 0;
}
