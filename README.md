# Big-Little Matching

This Rust crate implements a matching algorithm for a two-colored ranking graph. This crate also comes with an executable which can read it's input from CSV files.

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
