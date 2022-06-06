[![Rust](https://github.com/sugyan/yasai/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/sugyan/yasai/actions/workflows/rust.yml)

# YASAI: Yet Another Shogi library, for AI development

YASAI(野菜) is a Rust library for high-speed generation of legal moves and search for positions for Shogi.
It is based on the implementation of [`apery_rust`](https://github.com/HiraokaTakuya/apery_rust) and uses [`shogi_core`](https://github.com/rust-shogi-crates/shogi_core) as the fundamental types and functions.


## Examples
### Perft

```shell
cargo run --release --example perft 5
```
