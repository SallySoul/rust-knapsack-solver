# rust-knapsack-solver

This project is for CSE 5707 Fall 2022. The knapsack homework.

## Quick Start

Build with

```
cargo build --release
```

Solve an test case using the `solve` sub-command by either specifying a path with `-i` flag, or
piping the problem in through stdin

```
.target/release/rust-knapsack-solver solve -i path/to/test.txt
# OR
cat path/to/test.txt | ./rust-knapsack-solver solve
```

The tool and its two subcommands also support the `-h / --help` flags, which well display
information about additional options.

## Observations on Knapsack Problem Difficulty

Generally, we saw three ways to affect the difficulty of the problem, as decscribed in [4].
* The number of items
* The max / avg weight of the items
* The correlation of weight and value

## Notes on the Implementatation

The default solver used by the `solve` sub-command is based on the `minknap` algorithm [1]. This is
a dynamic programming based algorithm that incorporates several other observations / techniques.
The implementaion can be found in `src/solvers/minknap.rs`.

First, we note the dynamic programming implemenation needs to only keep track of one "layer" of states.
When we add an item, we need to modify this set of states to account for the additional item.
Our implemenation keeps a compressed version of the decision history using a tree containing unsigned 64 bit integers.
This way, only 1 bit per decision is needed, and states can share history in the tree.
This differs from other implementations of `minknap` that opt to recursivley solve smaller problem
and only track the recent decision history.

A useful concept when looking at the knapsack problem is the "break solution" and "break item".
The break solution is attained by greedily taking items as sorted by efficiency.
The first item that does not fit is the break item.
The motivator for `minknap` is that generally optimal solutions to the 0-1 knapsack problem differ only slightly from the break solution.
And where they differ is centered around the break item.
The naive dynamic programming implementation adds items one at a time, starting with no items.
`minknap` starts with the break solution and will iterativley try adding and removing items from that solution, in what Pisinger called the "expanding core".
Another difference between `minknap` and the naive dynamic solution is that states can be over capacity, since items can be removed later.

There are several ways that `minknap` bounds and discards states.
First, every state that is under capacity is a valid solution.
The most profitable valid state is our lower bound.
We can relax the integer constraint to gain a strong upper bound for the profit of a given state.
This is done by linearly adding / removing the next most efficient / inefficent item depending on whether the state is under of over capacity.
Lastly, we can remove so-called "dominated" states.
This is when we have a state with a lower profit at the same or higher weight than another known
state.

In addition to the paper, there are two existing implementations of Minknap that were helpful resources.
First, Pisenger shared a reference implementation for `minknap` written in C [2].
In addition, a C++ implementation that combines several techniques is available on github from user fontanf [3].

A major difference between our implementation and the ones above relates to sorting the items by efficiency.
The paper describes a technique for doing the minimal ammount of sorting necessary.
We opted to compleletly sort the items as this made the code far simpler and is relativley cheap computationally.

## Resources Used

[1] Pisinger, David (1997) "A Minimal Algorithm For The 0-1 Knapsack Problem"

[2] Pisinger, David (1997) "minkap.c" http://hjemmesider.diku.dk/~pisinger/minknap.c

[3] Fontanf (2022) "knapsacksolver" https://github.com/fontanf/knapsacksolver

[4] Pisinger, David (2004) "Where are the hard knapsack problems?"
