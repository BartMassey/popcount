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


/* HAKMEM 169 */
/* 9 ops plus divide, 2 long immediates, 8 stages */
static inline uint32_t
popcount_hakmem(uint32_t mask)
{
    uint32_t y;

    y = (mask >> 1) & 033333333333;
    y = mask - y - ((y >>1) & 033333333333);
    return ((y + (y >> 3)) & 030707070707) % 63;
}


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
    return n & 0x1f;
}


/* Try starting with a ternary stage for parallelism */
/* 21 ops, 2 long immediates, 14 stages */
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
    return n & 0x1f;
}


#define BLOCKSIZE 2048
uint32_t randoms[BLOCKSIZE];

uint32_t
drive_hakmem(int n) {
    int i, j;
    uint32_t result = 0;
    for (j = 0; j < n; j++)
	for (i = 0; i < BLOCKSIZE; i++)
	    result += popcount_hakmem(randoms[i]);
    return result;
}

uint32_t
drive_keane(int n) {
    int i, j;
    uint32_t result = 0;
    for (j = 0; j < n; j++)
	for (i = 0; i < BLOCKSIZE; i++)
	    result += popcount_keane(randoms[i]);
    return result;
}

uint32_t
drive_2(int n) {
    int i, j;
    uint32_t result = 0;
    for (j = 0; j < n; j++)
	for (i = 0; i < BLOCKSIZE; i++)
	    result += popcount_2(randoms[i]);
    return result;
}

uint32_t
drive_3(int n) {
    int i, j;
    uint32_t result = 0;
    for (j = 0; j < n; j++)
	for (i = 0; i < BLOCKSIZE; i++)
	    result += popcount_3(randoms[i]);
    return result;
}

struct drivers {
    char *name;
    uint32_t (*func)(int);
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


void
run_driver(struct drivers *d, int n) {
    struct timeval start, end;
    uint32_t result, elapsed;
    printf("preheating %s (%d)\n", d->name, d->func(10));
    assert(gettimeofday(&start, 0) != -1);
    result = d->func(n);
    assert(gettimeofday(&end, 0) != -1);
    elapsed = elapsed_msecs(&start, &end);
    printf("timed %s at %d msecs for %g nsecs/iter (%d)\n",
	   d->name, elapsed, elapsed * 1.0e6 / BLOCKSIZE * n, result);
}

struct drivers drivers[] = {
    {"popcount_hakmem", drive_hakmem},
    {"popcount_keane", drive_keane},
    {"popcount_2", drive_2},
    {"popcount_3", drive_3},
    {0, 0}
};

void
run_all(int n) {
    struct drivers *d;
    for (d = drivers; d->name; d++)
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
