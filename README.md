# Big-Little Matching

This Rust crate implements a stable matching algorithm for a set of assignments of littles to bigs. This crate also comes with an executable which can read it's input from CSV files.

## Algorithm

The matching algorithm considers two sets for matching, bigs and littles, of which many littles can be assigned to single big. The algorithm implemented here is a variant of the [Gale–Shapley Algorithm](https://en.wikipedia.org/wiki/Gale%E2%80%93Shapley_algorithm) but in the case of multiple assignments for a single big, and a variable carrying capacity for each big. 

First, the maximal matching is found which considers the preferences of each little from highest to lowest to find the first big which also has that little in their preferences. If this match is found, the little is assigned tentatively to that big. If no match is found, the little is left unmatched for the duration of the algorithm. After the maximal matching, the algorithm proceeds by considering the big with the largest assignment and takes the lowest ranking little on that assignment and finds the next big that it can match with. This continues until either:

1. All bigs have been matched with at least one little.

or if that never happens, 

2. All the matches have the same number of littles.

Then, the algorithm stops and returns the current matches, collecting the remaining bigs which are unmatched.

## How to Use

To compute a matching, replace `{BIGS}` and `{LITTLES}` with the paths associated to each CSV file in the following

```sh
cargo run --release --all-features {BIGS} {LITTLES}
```

The CSV header format that this executable accepts is as follows

```text
... , Name , Rank 1 , Rank 2 , Rank 3 , ...
```

where `Name` must appear with that exact spelling, and the rank headers can be any non-empty strings. Any columns before `Name` are ignored. There should be _no_ columns after `Name` which are not meant to represent preferences. Each entry in the two tables should be a name which uniquely identifies the matching participants. The parser ensures that individuals are asssigned to only one kind, `Big` or `Little`, and will return an error if it tries to assign one participant to two kinds.

## Documentation

To see the documentation for this crate run the following

```sh
cargo doc --all-features --open
```

and open the output file in the browser if it does not open automatically.

## License

This work is released into the public domain with CC0 1.0. Alternatively, it is licensed under the Apache License 2.0. See [`LICENSE`](./LICENSE) for more details.
