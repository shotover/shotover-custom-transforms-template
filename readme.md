# Shotover Custom Transforms Template

This repo is a template from which you can build your own shotover custom transform.
To get started just run `git clone https://github.com/shotover/shotover-custom-transforms-template`.

Use an example transform that matches the protocol you are working with as a base. e.g.

* `redis-get-rewrite` - for redis
* `kafka-fetch-rewrite` - for kafka

Run `cargo build --release` in the root of the repo to get a shotover binary containing your custom transform at `target/release/shotover-bin`.

The shotover-bin crate contains integration tests that will be useful for developing your transform.
Run `cargo test` in the root of the repo to run these tests.
