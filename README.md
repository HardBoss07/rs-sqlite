# rs-sqlite

An educational SQLite clone written in Rust.

The goal of this project is to dive deep into database storage engines and B-trees by following [cstack's tutorial](https://cstack.github.io/db_tutorial/). To add an extra layer of complexity, I am actively translating the original C codebase into Rust, navigating the architectural shift from manual memory allocation to the strict requirements of the Rust borrow checker.

**Architecture Note:** Instead of building a pointer-heavy, recursive memory graph (which fights Rust's borrow checker), this project implements its B-Tree using a flat, data-oriented architecture-manipulating raw byte-slices directly inside pre-allocated virtual disk pages to match on-disk binary layouts seamlessly.