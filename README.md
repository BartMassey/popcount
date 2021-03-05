# Popcount
Copyright &copy; 2007 Bart Massey  
2021-03-05

Here's some implementations of bit population count in C and
Rust, with benchmarks.

The writeup in this README is based on work from 2007 and
later, as far as I can tell today. It consists of status,
and then a chronological log including benchmark results.

## Background

Everybody on the net does bit population count.  I needed to
do it for [XCB](http://xcb.freedesktop.org).  My criteria
were:

  * Should be fast even on cruddy machines (so limit
    operators to integer ones and try to avoid multiply and
    especially divide).
  * Should be deterministically fast in the worst case.
  * Should be comprehensible / maintainable.

X has always used the venerable
[HAKMEM 169](http://www.inwap.com/pdp10/hbaker/hakmem/hacks.html#item169)
algorithm, but this didn't seem very good for the above
criteria.  [Keith Packard](http://keithp.com), who inserted
the HAKMEM code, tells me he selected it mainly because it
was the first reasonable thing he came across when he was
looking for an algorithm.
    
I wrote this code to benchmark some tries at a replacement.
The Wikipedia <a
href="http://en.wikipedia.org/wiki/Hamming_weight">page</a>
was a fairly good starting point.  This
[bonus material](http://www.hackersdelight.org/divcMore.pdf)
from *Hacker's Delight* was a useful reference. I'd lost my
copy of the book, but I've now replaced it and I can say
that there's nothing better in there (as of the second
edition), which is a pretty strong statement.

I think my code is also a reasonable tutorial for
micro-benchmarking frameworks, illustrating some reasonable
techniques. There's still some things I would fix, though:
see below.

## Build and Run

You can build the C code with both GCC and Clang and the
Rust code with Cargo by typing `make`, but you should
first examine the Makefile and set what you need to
tune. Depending on your environment, you might just want to
build specific binaries.

Once you've built `popcount_gcc` or whichever, run it with a
number of iterations to bench: 100000 is good to start. It
will spit out some explanatory numbers.

## Advice

* If you need predictable speed, use `__builtin_popcount()`
  (GCC, Clang) or `count_ones()` (Rust). Your machine might
  have a popcount instruction (most modern ones finally do),
  and this will use it if present.  C code will be portable
  only to GCC-compatible C compilers. Rust code will
  currently only get a speedup when compiled with
  `--cpu-type native`. All compilers bench reasonably fast
  on my box without a popcount instruction available; looks
  like they are using `popcount_2` or similar, but
  generating slightly better code. (I haven't investigated
  what they are actually doing.)

* If you want true machine-independence and don't care *too*
  much about speed, use the famous `popcount_2`. It's fast
  enough, familiar and completely portable.

* Use `popcount_tabular_8` if you're willing to burn some
  cache (not much) and want a slight but portable speedup
  over `popcount_2`. Also if you're on some machine without
  a barrel shifter, this can be adapted to not do any shifts
  without too much work at the expense of hitting
  memory. (Do byte reads from the input variable.)


## Future Improvements

* The op is called in a tight loop, so self-pipelining is
  in play.  This isn't very realistic.  However, it's hard
  to build a reasonable set of calling environments, and
  hard to calibrate out their effects.

* I only report elapsed time.  This is probably OK for
  this application, but it would be easy to add user and
  system time reporting.

* The user gives the number of iterations of a fixed-size
  block.  I should do the `x11perf` thing and have the user
  supply a desired number of seconds instead, and use the
  preheat loop to calibrate.

  For now, I have a set of manually-chosen loop divisors
  to speed up the slower algorithms. All algorithms take
  about the same amount of time per "iteration" specified
  on my machine, although probably not on yours.

## Talk

I have included [slides](pdxbyte-popcount.pdf) from a
[PDXByte talk](https://www.youtube.com/watch?v=opJvsJk1B68)
I gave in March 2014. Enjoy.

## License

This work is made available under the "MIT License". Please
see the file LICENSE in this distribution for license terms.

## Notes and Results

This is a log of work on the project.  Most recent numbers,
current entry, then older entries in chronological order.

### Benchmark Results

### 2021-03-05

Performance on Raspberry Pi 400 as of 2021-03-05. GCC
10.2.1, Clang 11.0.1-2, rustc 1.52.0-nightly
2021-03-04. Debian 5.10.0-3-arm64. <blockquote>

    $ sh run-bench.sh

    popcount_gcc
    popcount_naive: 6.25e+07 iters in 2652 msecs for 42.43 nsecs/iter
    popcount_8: 2.5e+08 iters in 2643 msecs for 10.57 nsecs/iter
    popcount_6: 2.5e+08 iters in 2365 msecs for 9.46 nsecs/iter
    popcount_hakmem: 2.5e+08 iters in 2919 msecs for 11.68 nsecs/iter
    popcount_keane: 2.5e+08 iters in 2780 msecs for 11.12 nsecs/iter
    popcount_anderson: 1.66666e+08 iters in 1854 msecs for 11.12 nsecs/iter
    popcount_3: 2.5e+08 iters in 2364 msecs for 9.46 nsecs/iter
    popcount_4: 2.5e+08 iters in 2226 msecs for 8.90 nsecs/iter
    popcount_2: 2.5e+08 iters in 2224 msecs for 8.90 nsecs/iter
    popcount_mult: 2.5e+08 iters in 3058 msecs for 12.23 nsecs/iter
    popcount_tabular_8: 2.5e+08 iters in 1670 msecs for 6.68 nsecs/iter
    popcount_tabular_16: 2.5e+08 iters in 4662 msecs for 18.65 nsecs/iter
    popcount_cc: 1e+09 iters in 12232 msecs for 12.23 nsecs/iter
    47127364941

    popcount_clang
    popcount_naive: 6.25e+07 iters in 2683 msecs for 42.93 nsecs/iter
    popcount_8: 2.5e+08 iters in 2226 msecs for 8.90 nsecs/iter
    popcount_6: 2.5e+08 iters in 2225 msecs for 8.90 nsecs/iter
    popcount_hakmem: 2.5e+08 iters in 2780 msecs for 11.12 nsecs/iter
    popcount_keane: 2.5e+08 iters in 2919 msecs for 11.68 nsecs/iter
    popcount_anderson: 1.66666e+08 iters in 1946 msecs for 11.68 nsecs/iter
    popcount_3: 2.5e+08 iters in 2223 msecs for 8.89 nsecs/iter
    popcount_4: 2.5e+08 iters in 2085 msecs for 8.34 nsecs/iter
    popcount_2: 2.5e+08 iters in 2224 msecs for 8.90 nsecs/iter
    popcount_mult: 2.5e+08 iters in 3058 msecs for 12.23 nsecs/iter
    popcount_tabular_8: 2.5e+08 iters in 1391 msecs for 5.56 nsecs/iter
    popcount_tabular_16: 2.5e+08 iters in 4652 msecs for 18.61 nsecs/iter
    popcount_cc: 1e+09 iters in 12231 msecs for 12.23 nsecs/iter
    47127364941

    popcount_rs
    popcount_naive: 6.25e7 iters in 2628 msecs for 42.04 nsecs/iter
    popcount_8: 2.5e8 iters in 2226 msecs for 8.91 nsecs/iter
    popcount_6: 2.5e8 iters in 2226 msecs for 8.90 nsecs/iter
    popcount_hakmem: 2.5e8 iters in 2780 msecs for 11.12 nsecs/iter
    popcount_keane: 2.5e8 iters in 2920 msecs for 11.68 nsecs/iter
    popcount_anderson: 1.66666e8 iters in 3522 msecs for 21.13 nsecs/iter
    popcount_3: 2.5e8 iters in 2224 msecs for 8.90 nsecs/iter
    popcount_4: 2.5e8 iters in 2085 msecs for 8.34 nsecs/iter
    popcount_2: 2.5e8 iters in 2224 msecs for 8.90 nsecs/iter
    popcount_mult: 2.5e8 iters in 3061 msecs for 12.24 nsecs/iter
    popcount_tabular_8: 2.5e8 iters in 13344 msecs for 53.37 nsecs/iter
    popcount_tabular_16: 2.5e8 iters in 11525 msecs for 46.10 nsecs/iter
    popcount_rs: 1e9 iters in 12245 msecs for 12.24 nsecs/iter
    47127364941

</blockquote>

The overall timings make sense inasmuch as the Raspberry Pi
is clocked about half as fast as the Intel box benchmarked
here.

Interestingly, of the non-tabular popcounts, `popcount_4` is
a noticeable winner over `popcount_2` here. Also of note is
the terrible performance of Rust on the tabular popcounts —
not sure what's going on there.

There's a `CNT` instruction on this hardware, but it's
provided by the Neon SIMD vector processor — the work to get
a 32-bit value out of a scalar register and into a vector
register, then get the count out, is substantial. As a
result, the compiler builtins are fairly dramatically worse
than `popcount_4`.

### 2020-02-25

Performance as of 2020-02-25.  GCC 9.2.1, Clang 8.0.1-7,
rustc 1.43.0-nightly 2020-02-24. Intel Core i7-4770K
(Haswell) CPU @ 3.50GHz. <blockquote>

    $ sh run-bench.sh

    popcount_gcc
    popcount_naive: 6.25e+07 iters in 929 msecs for 14.86 nsecs/iter
    popcount_8: 2.5e+08 iters in 1137 msecs for 4.55 nsecs/iter
    popcount_6: 2.5e+08 iters in 1096 msecs for 4.38 nsecs/iter
    popcount_hakmem: 2.5e+08 iters in 1361 msecs for 5.44 nsecs/iter
    popcount_keane: 2.5e+08 iters in 1427 msecs for 5.71 nsecs/iter
    popcount_anderson: 1.66666e+08 iters in 953 msecs for 5.72 nsecs/iter
    popcount_3: 2.5e+08 iters in 1088 msecs for 4.35 nsecs/iter
    popcount_4: 2.5e+08 iters in 1012 msecs for 4.05 nsecs/iter
    popcount_2: 2.5e+08 iters in 1040 msecs for 4.16 nsecs/iter
    popcount_mult: 2.5e+08 iters in 985 msecs for 3.94 nsecs/iter
    popcount_tabular_8: 2.5e+08 iters in 776 msecs for 3.10 nsecs/iter
    popcount_tabular_16: 2.5e+08 iters in 1703 msecs for 6.81 nsecs/iter
    popcount_cc: 1e+09 iters in 1294 msecs for 1.29 nsecs/iter
    popcount_x86: 1e+09 iters in 1292 msecs for 1.29 nsecs/iter
    50332463787

    popcount_clang
    popcount_naive: 6.25e+07 iters in 1025 msecs for 16.40 nsecs/iter
    popcount_8: 2.5e+08 iters in 1983 msecs for 7.93 nsecs/iter
    popcount_6: 2.5e+08 iters in 1084 msecs for 4.34 nsecs/iter
    popcount_hakmem: 2.5e+08 iters in 1336 msecs for 5.34 nsecs/iter
    popcount_keane: 2.5e+08 iters in 1259 msecs for 5.04 nsecs/iter
    popcount_anderson: 1.66666e+08 iters in 836 msecs for 5.02 nsecs/iter
    popcount_3: 2.5e+08 iters in 1027 msecs for 4.11 nsecs/iter
    popcount_4: 2.5e+08 iters in 1018 msecs for 4.07 nsecs/iter
    popcount_2: 2.5e+08 iters in 1040 msecs for 4.16 nsecs/iter
    popcount_mult: 2.5e+08 iters in 987 msecs for 3.95 nsecs/iter
    popcount_tabular_8: 2.5e+08 iters in 778 msecs for 3.11 nsecs/iter
    popcount_tabular_16: 2.5e+08 iters in 1664 msecs for 6.66 nsecs/iter
    popcount_cc: 1e+09 iters in 1294 msecs for 1.29 nsecs/iter
    popcount_x86: 1e+09 iters in 1297 msecs for 1.30 nsecs/iter
    50332463787

    popcount_rs
    popcount_naive: 6.25e7 iters in 1052 msecs for 16.84 nsecs/iter
    popcount_8: 2.5e8 iters in 2065 msecs for 8.26 nsecs/iter
    popcount_6: 2.5e8 iters in 1095 msecs for 4.38 nsecs/iter
    popcount_hakmem: 2.5e8 iters in 1345 msecs for 5.38 nsecs/iter
    popcount_keane: 2.5e8 iters in 1290 msecs for 5.16 nsecs/iter
    popcount_anderson: 1.66666e8 iters in 1170 msecs for 7.02 nsecs/iter
    popcount_3: 2.5e8 iters in 1030 msecs for 4.12 nsecs/iter
    popcount_4: 2.5e8 iters in 1009 msecs for 4.04 nsecs/iter
    popcount_2: 2.5e8 iters in 1039 msecs for 4.16 nsecs/iter
    popcount_mult: 2.5e8 iters in 989 msecs for 3.96 nsecs/iter
    popcount_tabular_8: 2.5e8 iters in 869 msecs for 3.48 nsecs/iter
    popcount_tabular_16: 2.5e8 iters in 1387 msecs for 5.55 nsecs/iter
    popcount_rs: 1e9 iters in 1297 msecs for 1.30 nsecs/iter
    popcount_x86: 1e9 iters in 1300 msecs for 1.30 nsecs/iter
    50332463787

</blockquote>

### 2020-02-25

Mostly Rust changes. I brought the Rust code up to 2018
Edition, fixed some silliness in the `Cargo.toml`, and added
the `Cargo.lock` for reproducibility. I also fixed a nit in
the driver macro that, while not performance-related,
reduced the code ugliness slightly.

Finally, I cleaned up the `README.md` a bit to reflect
current reality.

### 2018-02-18

I replaced the C and Rust standard random number generators
with a common linear congruential PRNG seeded
deterministically. This gives better run-to-run and
cross-implementation compatibility.

I re-added the long-lost Anderson popcount. There was an…
incident with commits in two different upstream repositories
for a while, and also the Rust implementation of Anderson is
tough because of casting issues. I suspect the Rust
performance could be improved, but it's a slow and
complicated popcount anyway and I can't bring myself to
care. Patches welcome.

After some work, the C and Rust versions run identical
benchmarks in identical order and produce an identical total
popcount. They are thus as comparable as I know how to make
them. I added a benchmarking shell script.

I reorganized this README to be in some sane order and
cleaned it up quite a lot.

I relicensed to MIT. This should make it easier for folks to
embed these popcounts in random open source things.

### 2007-11-27

Of the things I tried the one that seemed to be
second-fastest on my 2.13GHz Core II Duo (generously given
me by Intel&mdash;thanks much!) was<blockquote>

    /* Divide-and-conquer with a ternary stage to reduce masking */
    /* 17 ops, 2 long immediates, 12 stages */
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

</blockquote> This is a classic divide-and-conquer popcount.
The only strange twists are the initial ternary stage, which
gets rid of a masking stage by making room a little earlier,
and the final ternary stage, which gives a few more
available ops to a superscalar machine at the end.  There's
also the standard clever trick to save a mask and op in the
first stage.

The slightly strange thing was that the classic binary
divide-and-conquer straight off Wikipedia was quite a bit
faster.<blockquote>

    /* Classic binary divide-and-conquer popcount.
       This is popcount_2() from
       http://en.wikipedia.org/wiki/Hamming_weight */
    /* 15 ops, 3 long immediates, 14 stages */
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

</blockquote>Why was this strange?  My CPU should certainly
have been able to dispatch 4 arithmetic ops per cycle and
keep everything in registers.  Thus, the limiting factor
should be either the number of pipeline stages used or the
number of immediate loads performed.  However, the ternary
algorithm has fewer of both of these than the binary one.  I
fear that the timing loop may be interfering with my
results, or that the compiler or the CPU may be losing on
dispatch.

The actual performance numbers on my Core II duo were:<blockquote>

    $ ./popcount 1000000
    popcount_naive: 1.6666e+07 iters in 688 msecs for 41.28 nsecs/iter
    popcount_8: 1e+08 iters in 995 msecs for 9.95 nsecs/iter
    popcount_6: 2e+08 iters in 1725 msecs for 8.625 nsecs/iter
    popcount_hakmem: 2e+08 iters in 1411 msecs for 7.055 nsecs/iter
    popcount_keane: 1e+09 iters in 6566 msecs for 6.566 nsecs/iter
    popcount_3: 1e+09 iters in 6270 msecs for 6.27 nsecs/iter
    popcount_2: 1e+09 iters in 5658 msecs for 5.658 nsecs/iter
    popcount_mult: 1e+09 iters in 4169 msecs for 4.169 nsecs/iter

</blockquote>These numbers overall compare quite well with
what you would predict based on the number of pipeline
stages and arithmetic ops in the algorithms; they say
astounding things about the hundreds of picoseconds
(picoseconds!) an individual operation takes on a modern
processor.  While it isn't suitable for my immediate needs,
check out the multiply-based implementation, which uses the
multiply for the accumulation step at the end.  Amazing that
it's faster than a couple of shifts and adds on Intel's
current flagship parts.

### 2007-12

I had said "Now that I think about it, though,
what's more likely is that the shifts by small powers of two
are being done in the address unit as per standard Intel
practice, where shifts by multiples of three need to be done
on the ALU, reducing the number of pipeline stages for the
binary version substantially."  It turns out I was wrong;
Mike Haertel showed me that the number of instructions is
all that will matter here on an Intel processor because of
false dependencies.)

On a superscalar RISC processor, I'd actually expect the
first algorithm to do better.

### 2008-04-30

At the suggestion of Bill Trost, I added 8 and 16-bit
table-driven popcount implementations.  These perform the
fastest in the benchmark test-stand, at about 3ns/iteration.
They're about the same speed, so one would obviously use the
8-bit version.

The problem with the table-driven speed is that I don't
believe it.  It's amazing that Intel's L1 cache is so cheap.
However, one can expect that the table lookup would
occasionally miss this small cache during normal usage,
which would be devastating to the performance of this
approach.  The tight loop here is guaranteeing us that the
table becomes locked in cache.  Maybe when I get some more
energy I'll create a more realistic testbench, and we'll see
how the table-driven algorithms compare to the
pure-computational ones in 2008.

### 2009-04-07

At the suggestion of David Taht, Mike Haertel and I
investigated the anomalously good performance of the
divide-and-conquer popcounts on some 64-bit machines running
gcc 3.4.  Turns out that this version of gcc is smart enough
to notice that the original version of the popcount code
didn't have a loop dependency, and that all 64-bit machines
have SSE: it was running four iterations of popcount in
parallel on the MMX unit. :-) Fixed by introducing the
obvious dependency.

### 2014-03-09

Here's the numbers for my modern Haswell machine (Intel Core
i7-4770K CPU @ 3.50GHz), compiled with Debian GCC
4.8.2 `-O2 -march=core-avx2`.<blockquote>

    bartfan @ ./popcount 1000000
    popcount_naive: 1.25e+08 iters in 2070 msecs for 16.56 nsecs/iter
    popcount_8: 1e+09 iters in 4807 msecs for 4.81 nsecs/iter
    popcount_6: 1e+09 iters in 4569 msecs for 4.57 nsecs/iter
    popcount_hakmem: 1e+09 iters in 5419 msecs for 5.42 nsecs/iter
    popcount_keane: 1e+09 iters in 5674 msecs for 5.67 nsecs/iter
    popcount_3: 1e+09 iters in 4037 msecs for 4.04 nsecs/iter
    popcount_4: 1e+09 iters in 3958 msecs for 3.96 nsecs/iter
    popcount_2: 1e+09 iters in 4134 msecs for 4.13 nsecs/iter
    popcount_mult: 1e+09 iters in 3916 msecs for 3.92 nsecs/iter
    popcount_tabular_8: 1e+09 iters in 3356 msecs for 3.36 nsecs/iter
    popcount_tabular_16: 1e+09 iters in 4372 msecs for 4.37 nsecs/iter

</blockquote>Except for `popcount_naive` (16.68 nsecs/iter) and
`popcount_8` (4.80 nsecs/iter) these timings were reproduced
exactly on a run ten times as long. Bumping the optimization
level to `-O4` made only small differences in these, and
subsequently removing the Haswell-specific optimizations
made little difference at all.<blockquote>

    popcount_naive: 1.25e+08 iters in 2079 msecs for 16.63 nsecs/iter
    popcount_8: 1e+09 iters in 4540 msecs for 4.54 nsecs/iter
    popcount_6: 1e+09 iters in 4298 msecs for 4.30 nsecs/iter
    popcount_hakmem: 1e+09 iters in 5430 msecs for 5.43 nsecs/iter
    popcount_keane: 1e+09 iters in 5683 msecs for 5.68 nsecs/iter
    popcount_3: 1e+09 iters in 4030 msecs for 4.03 nsecs/iter
    popcount_4: 1e+09 iters in 3967 msecs for 3.97 nsecs/iter
    popcount_2: 1e+09 iters in 4151 msecs for 4.15 nsecs/iter
    popcount_mult: 1e+09 iters in 3921 msecs for 3.92 nsecs/iter
    popcount_tabular_8: 1e+09 iters in 3365 msecs for 3.37 nsecs/iter
    popcount_tabular_16: 1e+09 iters in 4386 msecs for 4.39 nsecs/iter

</blockquote>In general, compiling with CLANG made the slow
benchmarks a little faster, and the fast ones slightly
slower. The most important difference was probably
`popcount_tabular_8`, which sped to 3.16 nsecs/iter.

Note that most of the conclusions of 2009
remain roughly valid.  Naive is still as terrible as one
would expect. Most everything else is more-or-less tied,
except 8-bit tabular, which is some 25% faster. Note that
16-bit tabular is losing substantially now, and that the
divide-and-conquer popcounts have pretty much caught up with
multiplication.

### 2016-02-21

I found out that my previously-benchmarked Haswell machine
(Intel Core i7-4770K CPU @ 3.50GHz) has a `popcnt`
instruction. Who knew?  I added both a call to
`__builtin_popcount()` and an inline assembly `popcnt`
version to the benchmarks.  Here's all the numbers again.
<blockquote>

    popcount_naive: 1.25e+08 iters in 2076 msecs for 16.61 nsecs/iter
    popcount_8: 1e+09 iters in 4520 msecs for 4.52 nsecs/iter
    popcount_6: 1e+09 iters in 4525 msecs for 4.53 nsecs/iter
    popcount_hakmem: 1e+09 iters in 5622 msecs for 5.62 nsecs/iter
    popcount_keane: 1e+09 iters in 5791 msecs for 5.79 nsecs/iter
    popcount_3: 1e+09 iters in 4141 msecs for 4.14 nsecs/iter
    popcount_4: 1e+09 iters in 4177 msecs for 4.18 nsecs/iter
    popcount_2: 1e+09 iters in 4215 msecs for 4.21 nsecs/iter
    popcount_mult: 1e+09 iters in 4014 msecs for 4.01 nsecs/iter
    popcount_tabular_8: 1e+09 iters in 3417 msecs for 3.42 nsecs/iter
    popcount_tabular_16: 1e+09 iters in 4449 msecs for 4.45 nsecs/iter
    popcount_cc: 1e+09 iters in 1308 msecs for 1.31 nsecs/iter
    popcount_x86: 1e+09 iters in 1313 msecs for 1.31 nsecs/iter

</blockquote>
I am a little mystified at the (replicable) difference
between these runs and the previous 2014 runs on the same
machine. Obviously, we must take this whole thing with a
large grant of salt. The most likely difference is the
compiler: this is GCC 5.3.1, which seems to do a bit worse
overall than the 2014 version. Or something.

To clear things up a bit, I also ran with clang-3.8. Here's
that:
<blockquote>

    popcount_naive: 1.25e+08 iters in 2069 msecs for 16.55 nsecs/iter
    popcount_8: 1e+09 iters in 4805 msecs for 4.80 nsecs/iter
    popcount_6: 1e+09 iters in 4408 msecs for 4.41 nsecs/iter
    popcount_hakmem: 1e+09 iters in 5631 msecs for 5.63 nsecs/iter
    popcount_keane: 1e+09 iters in 5108 msecs for 5.11 nsecs/iter
    popcount_3: 1e+09 iters in 4162 msecs for 4.16 nsecs/iter
    popcount_4: 1e+09 iters in 4137 msecs for 4.14 nsecs/iter
    popcount_2: 1e+09 iters in 4224 msecs for 4.22 nsecs/iter
    popcount_mult: 1e+09 iters in 4028 msecs for 4.03 nsecs/iter
    popcount_tabular_8: 1e+09 iters in 3167 msecs for 3.17 nsecs/iter
    popcount_tabular_16: 1e+09 iters in 4040 msecs for 4.04 nsecs/iter
    popcount_cc: 1e+09 iters in 1308 msecs for 1.31 nsecs/iter
    popcount_x86: 1e+09 iters in 1314 msecs for 1.31 nsecs/iter

</blockquote>
Similar, but not the same. Conclusion: it's all in the compiler noise.

Having said that, check out that tasty `popcnt` instruction!
We've always said it doesn't really matter -- and it
doesn't, really. But that's pretty clearly the right answer
if GCC could figure out how to use it here without help from
inlining.

### 2016-02-24

Noticed that the `-march` flag should have been set
to `native`.<blockquote>

    popcount_naive: 6.25e+08 iters in 10333 msecs for 16.53 nsecs/iter
    popcount_8: 2.5e+09 iters in 11225 msecs for 4.49 nsecs/iter
    popcount_6: 2.5e+09 iters in 11162 msecs for 4.46 nsecs/iter
    popcount_hakmem: 2.5e+09 iters in 13846 msecs for 5.54 nsecs/iter
    popcount_keane: 2.5e+09 iters in 14281 msecs for 5.71 nsecs/iter
    popcount_3: 2.5e+09 iters in 10171 msecs for 4.07 nsecs/iter
    popcount_4: 2.5e+09 iters in 10290 msecs for 4.12 nsecs/iter
    popcount_2: 2.5e+09 iters in 10378 msecs for 4.15 nsecs/iter
    popcount_mult: 2.5e+09 iters in 9859 msecs for 3.94 nsecs/iter
    popcount_tabular_8: 2.5e+09 iters in 8431 msecs for 3.37 nsecs/iter
    popcount_tabular_16: 2.5e+09 iters in 11004 msecs for 4.40 nsecs/iter
    popcount_cc: 1e+10 iters in 12936 msecs for 1.29 nsecs/iter
    popcount_x86: 1e+10 iters in 12942 msecs for 1.29 nsecs/iter


</blockquote>
Fixing the architecture flattened things out a bit more, but
didn't change results in any interesting way. Basically, on
this machine and with current GCC, all the routines run at
the same speed except *naive,* which is expectedly slow, and
the inline `popcnt` instruction ones, which are expectedly
fast.

### 2018-01-30

Added a Rust implementation. It tries to conform to the C
implementation as much as reasonably feasible. The code
bodies are essentially copy-pasted.

This code currently needs nightly to get inline assembly
support: the x86 popcount will be compiled-out on non-x86,
and will cause a runtime assertion failure on x86 without
popcnt.

The native Rust `u32::count_ones()` method is not compiled
into a native `popcnt` instruction in current incarnations
unless you play games with `RUSTFLAGS`. `count_ones()` is
about the same speed as all the other fast popcounts,
though, so it's probably the right thing if you don't need
the machine instruction's speed.

Current C numbers, GCC 7.2.0 on my Haswell box described
above:
<blockquote>

    popcount_naive: 6.25e+07 iters in 930 msecs for 14.88 nsecs/iter
    popcount_8: 2.5e+08 iters in 1149 msecs for 4.60 nsecs/iter
    popcount_6: 2.5e+08 iters in 1111 msecs for 4.44 nsecs/iter
    popcount_hakmem: 2.5e+08 iters in 1455 msecs for 5.82 nsecs/iter
    popcount_keane: 2.5e+08 iters in 1512 msecs for 6.05 nsecs/iter
    popcount_3: 2.5e+08 iters in 1063 msecs for 4.25 nsecs/iter
    popcount_4: 2.5e+08 iters in 1034 msecs for 4.14 nsecs/iter
    popcount_2: 2.5e+08 iters in 1043 msecs for 4.17 nsecs/iter
    popcount_mult: 2.5e+08 iters in 994 msecs for 3.98 nsecs/iter
    popcount_tabular_8: 2.5e+08 iters in 856 msecs for 3.42 nsecs/iter
    popcount_tabular_16: 2.5e+08 iters in 1095 msecs for 4.38 nsecs/iter
    popcount_cc: 1e+09 iters in 1306 msecs for 1.31 nsecs/iter
    popcount_x86: 1e+09 iters in 1302 msecs for 1.30 nsecs/iter

</blockquote>

Clang 4.0 is probably more comparable because LLVM:
<blockquote>

    popcount_naive: 6.25e+07 iters in 992 msecs for 15.87 nsecs/iter
    popcount_8: 2.5e+08 iters in 1207 msecs for 4.83 nsecs/iter
    popcount_6: 2.5e+08 iters in 1097 msecs for 4.39 nsecs/iter
    popcount_hakmem: 2.5e+08 iters in 1351 msecs for 5.40 nsecs/iter
    popcount_keane: 2.5e+08 iters in 1266 msecs for 5.06 nsecs/iter
    popcount_3: 2.5e+08 iters in 1041 msecs for 4.16 nsecs/iter
    popcount_4: 2.5e+08 iters in 1022 msecs for 4.09 nsecs/iter
    popcount_2: 2.5e+08 iters in 1048 msecs for 4.19 nsecs/iter
    popcount_mult: 2.5e+08 iters in 999 msecs for 4.00 nsecs/iter
    popcount_tabular_8: 2.5e+08 iters in 779 msecs for 3.12 nsecs/iter
    popcount_tabular_16: 2.5e+08 iters in 994 msecs for 3.98 nsecs/iter
    popcount_cc: 1e+09 iters in 1304 msecs for 1.30 nsecs/iter
    popcount_x86: 1e+09 iters in 1308 msecs for 1.31 nsecs/iter

</blockquote>

Current Rust numbers, rustc 1.24.0 nightly 2018-01-01:
<blockquote>

    popcount_naive: 6.25e7 iters in 971 msecs for 15.54 nsecs/iter
    popcount_8: 2.5e8 iters in 1205 msecs for 4.82 nsecs/iter
    popcount_6: 2.5e8 iters in 1097 msecs for 4.39 nsecs/iter
    popcount_hakmem: 2.5e8 iters in 1349 msecs for 5.40 nsecs/iter
    popcount_keane: 2.5e8 iters in 1260 msecs for 5.04 nsecs/iter
    popcount_3: 2.5e8 iters in 1060 msecs for 4.24 nsecs/iter
    popcount_4: 2.5e8 iters in 1035 msecs for 4.14 nsecs/iter
    popcount_2: 2.5e8 iters in 1046 msecs for 4.18 nsecs/iter
    popcount_mult: 2.5e8 iters in 996 msecs for 3.98 nsecs/iter
    popcount_tabular_8: 2.5e8 iters in 837 msecs for 3.35 nsecs/iter
    popcount_tabular_16: 2.5e8 iters in 1211 msecs for 4.84 nsecs/iter
    popcount_x86: 1e9 iters in 1305 msecs for 1.31 nsecs/iter
    popcount_rs: 2.5e8 iters in 998 msecs for 3.99 nsecs/iter

</blockquote>

Up to variances, they're pretty much all the same.
