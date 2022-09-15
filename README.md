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
./target/release/rust-knapsack-solver solve -i test_assets/assignment_example.txt
# OR
cat test_assets/assignment_example.txt | ./target/release/rust-knapsack-solver solve
```

Note that printing the decision vector for large problems can take some time. To skip this (the decision vector will still be calculated and validated), pass `-n / --no-print-solution` as well.

The tool and its two subcommands also support the `-h / --help` flags, which will display
information about additional options.

There are additional tests in `test_assets`.

## Observations on Knapsack Problem Difficulty

Generally, we saw three ways to affect the difficulty of the problem, as decscribed in [4].
* The number of items
* The max / avg weight of the items
* The correlation of weight and value

Below is a table show the run time, memory used, and states explored for all the cases in `test_assets`. Our hardest test case in the suite has 30k items, but the weights range from 1 to 1 million, and the values are strongly correlated in that `v_i = w_i + 100k`.

From our experience, the single most important factor in runtime is reducing the states explored. A debug build of our current solver handily beats an earlier release build that did not remove dominated states. In addition, an earlier version of the build used a hashmap instead of keeping the states in an orderd buffer, which results in hasing operations consuming ~90% of the runtime.

```
Name                                  Time     States Explored   Mem Used
all_items_edge_case                   0.07s                      
assignment_example                    0.00s    110               2.86 kB
greedy_sol_500k                       0.11s    11696180          108.77 MB
item_larger_than_capacity_edge_case   0.00s    110               2.89 kB
large_item_edge_case                  0.00s    110               2.89 kB
random_400k                           0.09s    6849858           26.83 MB
strong_10k_1mw                        4.98s    1054352653        2.28 GB
strong_1k_100kw                       0.00s    79713             2.15 MB
strong_1k_1mw                         0.00s    163942            4.24 MB
strong_250k_100kw                     3.00s    693312841         817.75 MB
strong_30k_1mw                        25.52s   5788085093        9.67 GB
strong_500k_100kw                     9.72s    2278714198        2.17 GB
subset_sum                            0.00s    3449              98.81 kB
too_big_edge_case                     0.00s    55                3.67 kB
zero_weight_items                     0.00s    110               2.91 kB
```

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
At any given point in time, the most profitable valid solution is our lower bound.
We can relax the integer decision constraint to gain a strong upper bound for the profit of a given state.
This is done by linearly adding / removing the next most efficient / inefficent item depending on whether the state is under / over capacity.
Lastly, we can remove so-called "dominated" states.
This is when we have a state with a lower profit at the same or higher weight than another known
state.

In addition to the paper, there are two existing implementations of Minknap that were helpful resources.
First, Pisinger shared a reference implementation for `minknap` written in C [2].
In addition, a C++ implementation that combines several techniques is available on github from user fontanf [3].

A major difference between our implementation and the ones above relates to sorting the items by efficiency.
The paper describes a technique for doing the minimal ammount of sorting necessary.
We opted to compleletly sort the items as this made the code far simpler and is relativley cheap computationally.

We do make minor use of variable reduction.
We remove items with weights larger than the problem capacity.
We also automatically use items with zero weight.
Future work for this solver could include making more extensive use of variable reduction.
There is a body of work for this on the knapsack problem, and it is a technique relied on heavily in [3].

## Resources Used

[1] Pisinger, David (1997) "A Minimal Algorithm For The 0-1 Knapsack Problem"

[2] Pisinger, David (1997) "minkap.c" http://hjemmesider.diku.dk/~pisinger/minknap.c

[3] Fontanf (2022) "knapsacksolver" https://github.com/fontanf/knapsacksolver

[4] Pisinger, David (2004) "Where are the hard knapsack problems?"
