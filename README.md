# pcbrepair-rs

_pcbrepair_ is a Rust library for parsing ASUS "PCBRepair Tool" `.FZ` and ASRock "PCBRepair Pro" `.CAE` boardview files.

[![The version of the pcbrepair crate on crates.io](https://img.shields.io/crates/v/pcbrepair)](https://crates.io/crates/pcbrepair)


## Installation

```shell
cargo add pcbrepair
```


## Examples

Extract parts from `boardview.fz` as KiCad footprints into `boardview.pretty`:

```shell
cargo run --release --example fpextract boardview.fz
```

This also works for `.cae` files.


## License

_pcbrepair_ is published under the terms of the [GNU General Public License, version 3 or later](COPYING.txt).
