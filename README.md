# Big-Little Matching

This Rust crate implements a matching algorithm for a two-colored ranking graph. This crate also comes with a binary which can read it's input from CSV files.

## How to Use

To compute a matching, replace `{BIGS}` and `{LITTLES}` with the paths associated to each CSV file in the following:

```sh
cargo run --release --all-features {BIGS} {LITTLES}
```

## Documentation

To see the documentation for this crate run the following:

```sh
cargo doc --all-features --open
```

and open the output file in the browser if it does not open automatically.

## License

This work is released into the public domain with CC0 1.0. Alternatively, it is licensed under the Apache License 2.0. See [`LICENSE`](./LICENSE) for more details.
