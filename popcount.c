/*
 * Copyright (C) 2007 Bart Massey
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
#define BLOCKSIZE 2048
uint32_t randoms[BLOCKSIZE];

/* XXX Because the popcount routine wants to be inlined
   in the loop, we need to expand each popcount routine
   in its own driver. */
#define DRIVER(NAME) \
  uint32_t                                          \
  drive_##NAME(int n) {                             \
    int i, j;                                       \
    uint32_t result = 0;                            \
    for (j = 0; j < n; j++)                         \
	for (i = 0; i < BLOCKSIZE; i++)             \
	    result += popcount_##NAME(randoms[i]);  \
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


/* Single mask binary divide-and-conquer. */
/* 23 ops, 1 long immediate, 20 stages */
static inline uint32_t
popcount_2(uint32_t n)
{
    uint32_t m4 = 0x00ff00ff;
    uint32_t m3 = m4 ^ (m4 << 4);
    uint32_t m2 = m3 ^ (m3 << 2);
    uint32_t m1 = m2 ^ (m2 << 1);
    n = (n & m1) + ((n >> 1) & m1);
    n = (n & m2) + ((n >> 2) & m2);
    n = (n & m3) + ((n >> 4) & m3);
    n = (n & m4) + ((n >> 8) & m4);
    n += n >> 16;
    return n & 0x3f;
}
DRIVER(2)


/* Joe Keane, sci.math.num-analysis, 9 July 1995,
   as cited by an errata to Hacker's Delight.
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


/* Try starting with a ternary stage to reduce masking */
/* 18 ops, 2 long immediates, 16 stages */
static inline uint32_t
popcount_3(uint32_t n)
{
    uint32_t m1 = 011111111111;
    uint32_t m2 = 030707070707;
    n = (n & m1) + ((n >> 1) & m1) + ((n >> 2) & m1);
    n = (n & m2) + ((n >> 3) & m2);
    n += n >> 6;
    n += n >> 12;
    n += n >> 24;
    return n & 0x3f;
}
DRIVER(3)


struct drivers {
    char *name;
    uint32_t (*f)(uint32_t);
    uint32_t (*blockf)(int);
};

struct drivers drivers[] = {
    {"popcount_naive", popcount_naive, drive_naive},
    {"popcount_8", popcount_8, drive_8},
    {"popcount_6", popcount_6, drive_6},
    {"popcount_hakmem", popcount_hakmem, drive_hakmem},
    {"popcount_2", popcount_2, drive_2},
    {"popcount_keane", popcount_keane, drive_keane},
    {"popcount_3", popcount_3, drive_3},
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
test_driver(struct drivers *d, int n) {
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
    struct timeval start, end;
    uint32_t result, elapsed;
    printf("preheating %s (%d)\n", d->name, d->blockf(1000));
    assert(gettimeofday(&start, 0) != -1);
    result = d->blockf(n);
    assert(gettimeofday(&end, 0) != -1);
    elapsed = elapsed_msecs(&start, &end);
    printf("timed %s at %d msecs for %g nsecs/iter (%d)\n",
	   d->name, elapsed, elapsed * 1.0e6 / BLOCKSIZE / n, result);
}


void
run_all(int n) {
    struct drivers *d;
    for (d = drivers; d->name; d++)
	test_driver(d, n);
    for (d = drivers; d->name; d++)
	if (d->blockf)
	    run_driver(d, n);
}


int
main(int argc,
     char **argv) {
    extern long atoi();
    int n = atoi(argv[1]);
    init_randoms();
    run_all(n);
    return 0;
}
