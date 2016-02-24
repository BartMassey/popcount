# Popcount
Copyright &copy; 2016 Bart Massey  
2016-02-20

Here's some implementations of bit population count with
benchmarks. You can build `popcount` by typing `make`, but
you should first examine the Makefile and set what you need
to tune. You'll also need to install
[Nickle](http://nickle.org) to build the tables.

Once you've built `popcount`, run it with a number of
iterations to bench: 10000 is good to start. It will
spit out some explanatory numbers.

## Background

[This writeup is based on work from 2009-04-07 and earlier.]

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
algorithm, but this doesn't seem very good for the above
criteria.  [Keithp](http://keithp.com), who inserted the
HAKMEM code, tells me he selected it mainly because it was
the first reasonable thing he came across when he was
looking for an algorithm.
    
I wrote this code to benchmark some tries at a replacement.
The Wikipedia <a
href="http://en.wikipedia.org/wiki/Hamming_weight">page</a>
is a fairly good starting point.  This
[bonus material](http://www.hackersdelight.org/divcMore.pdf)
from *Hacker's Delight* was a useful reference. I'd lost my
copy of the book, but I've now replaced it and I can say
that there's nothing better in there (as of the first
edition: I do not yet have the second), which is a pretty
strong statement.

I think my code is also a reasonable tutorial for
benchmarking frameworks for C, illustrating some reasonable
techniques.  There's still some things I would fix, though:
Right now:

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
    preheat loop to calibrate.  (Update: I now do a sort of
    cheesy approximation to this.  See the source.)

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

(Update: I had said "Now that I think about it, though,
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

The actual performance numbers on my Core II duo in 2007 were:<blockquote>

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

**Update:** At the suggestion of Bill Trost, I added 8 and
16-bit table-driven popcount implementations.  These perform
the fastest in the benchmark test-stand, at about
3ns/iteration.  They're about the same speed, so one would
obviously use the 8-bit version.

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

**Update:** At the suggestion of David Taht, Mike Haertel and I
investigated the anomalously good performance of the
divide-and-conquer popcounts on some 64-bit machines running
gcc 3.4.  Turns out that this version of gcc is smart enough
to notice that the original version of the popcount code
didn't have a loop dependency, and that all 64-bit machines
have SSE: it was running four iterations of popcount in
parallel on the MMX unit. :-) Fixed by introducing the
obvious dependency.

**Update 2014-03-09:** Here's the numbers for my modern
Haswell machine (Intel Core i7-4770K CPU @ 3.50GHz).
<blockquote>

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
exactly on a run ten times as long.

Note that most of the conclusions of 2009 remain roughly
valid.  Naive is still as terrible as one would expect. Most
everything else is more-or-less tied, except 8-bit tabular,
which is some 25% faster. Note that 16-bit tabular is losing
substantially now, and that the divide-and-conquer popcounts
have pretty much caught up with multiplication.

**Update 2016-02-21:** I found out that my
previously-benchmarked Haswell machine (Intel Core i7-4770K
CPU @ 3.50GHz) has a `popcnt` instruction. Who knew?  I
added both a call to `__builtin_popcount()` and an inline
assembly `popcnt` version to the benchmarks.  Here's all the
numbers again.
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

**Update 2016-02-24:** Noticed that the `-march` flag should
have been set to `native`.
<blockquote>

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

Advice at this point:

* If you want machine-independent and don't care *too* much
  about speed, use the famous `popcount_2`. It's fast
  enough, familiar and truly portable.

* Use `popcount_tabular_8` if you're willing to burn some
  cache (not much) and want a slight but portable
  speedup. Also if you're on some machine without a barrel
  shifter, this can be adapted to not do any shifts without
  too much work at the expense of hitting cache. (Do
  byte reads from the input variable.)

* If you need all the speed, and your machine has a popcount
  instruction, use `popcount_cc`. Code is still portable,
  but only to GCC-compatible compilers (GCC, Clang). I don't
  know how fast it will be on machines without a popcount
  instruction.

-----

I have included [slides](pdxbyte-popcount.pdf) from a
[PDXByte talk](https://www.youtube.com/watch?v=opJvsJk1B68)
I gave in March 2014. Enjoy.

-----

This work is made available under the GPL version 3 or
later. Please see the file COPYING in this distribution for
license terms.
