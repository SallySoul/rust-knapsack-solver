# rust-knapsack-solver

This project is for CSE 5707 Fall 2022. The knapsack homework.

## Quick Start

Get source with

```
git clone
```

Build with

```
cargo build --release
```

Solve an test case with

```
./rust-knapsack-solver solve -i path/to/test.txt
# OR
cat path/to/test.txt | ./rust-knapsack-solver solve
```

## How to Build

You will need the rust toolchain available.
The officially recommended method for handling rust toolchains is `rustup`

Once you have installed rust, navigate to the root of the repo and you can use
`cargo run` to try it out. Note that arguments intened for the tool must be passed after `--` so that cargo run doesn't grab them.

```
cargo run --release -- --help
```

Both `cargo build --release` and `cargo run --release` put the target executable in `target/release`, so you could also do something like

```
cargo build --release &&
./target/release/rust-knapsack-solver --help
```

## How to Use

The tool includes two sub-commands, `generate` and `solve`

### Generate Test Cases

```
$ ./rust-knapsack-solver generate --help
rust-knapsack-solver-generate 

USAGE:
    rust-knapsack-solver generate [OPTIONS] --output-path <OUTPUT_PATH>

OPTIONS:
    -c, --capacity <CAPACITY>
            Capacity for the knapsack [default: 700]

        --coeff <COEFF>
            [default: 0.1]

        --correlation <CORRELATION>
            How the weight and values of the items should correlate. Options are None, and Some If
            choosing Some, use the coeff argument to determine ammount of correlation [default:
            None]

    -h, --help
            Print help information

    -n, --item-count <ITEM_COUNT>
            How many items to generate [default: 30]

    -o, --output-path <OUTPUT_PATH>
            Where to write the problem file

    -v, --value-bound <VALUE_BOUND>
            Upper bound on weight [default: 100]

    -w, --weight-bound <WEIGHT_BOUND>
            Upper bound on weight [default: 100]
```

### Solve Test Cases

```
$ ./rust-knapsack-solver solve --help
rust-knapsack-solver-solve 

USAGE:
    rust-knapsack-solver solve [OPTIONS]

OPTIONS:
    -h, --help                       Print help information
    -i, --input-file <INPUT_FILE>    Problem file to try. If not specified, problem should be fed in
                                     via STD IN
    -s, --solver <SOLVER>            Which solver implementation to use [default: Greedy]
```
