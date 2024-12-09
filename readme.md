# Shotover Custom Transforms Template

[![dependency status](https://deps.rs/repo/github/shotover/shotover-custom-transforms-template/status.svg)](https://deps.rs/repo/github/shotover/shotover-custom-transforms-template)

This repo is a template from which you can build your own shotover custom transform.
To get started just run `git clone https://github.com/shotover/shotover-custom-transforms-template`.

Use an example transform that matches the protocol you are working with as a base. e.g.

* `valkey-get-rewrite` - for valkey
* `kafka-fetch-rewrite` - for kafka

Run `cargo build --release` in the root of the repo to get a shotover binary containing your custom transform at `target/release/shotover-bin`.

The shotover-bin crate contains integration tests that will be useful for developing your transform.
Run `cargo test` in the root of the repo to run these tests.
