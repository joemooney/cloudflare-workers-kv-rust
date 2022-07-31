# A Simple cloud-based Key-Value Database

<sub>last updated Feb 6, 2022</sub>

Create a KV store hosted on cloudflare (about 100K transactions a month for free!).

Follow [this tutorial](https://developers.cloudflare.com/workers/tutorials/workers-kv-from-rust)

## Goal

Provide simple set/get endpoints to update valies in a key/value store:

* `curl -X PUT 'localhost:8787/foo?value=bar'`
* `curl 'localhost:8787/foo'` return "bar"
* `curl --header "Content-Type: Application/jsoN" -X POST 'localhost:8787/foo?value=bar' -d '{"product_id": 123456, "quantitY": 100}'`
* `curl -X GET localhost:8787/datum/3`

1. `wrangler build`
2. `wrangler dev`  (now you can use the localhost 8787 endpoints)
3. `wrangler publish`  (now anyone can use the published endpoints: `https://workers-kv-from-rust.joemooney.workers.dev/foo`

## Getting Started

A template for kick starting a Cloudflare worker project using [`workers-rs`](https://github.com/cloudflare/workers-rs).

This template is designed for compiling Rust to WebAssembly and publishing the resulting worker to
Cloudflare's [edge infrastructure](https://www.cloudflare.com/network/).

## Usage

This template starts you off with a `src/lib.rs` file, acting as an entrypoint for requests hitting your Worker.
Feel free to add more code in this file, or create Rust modules anywhere else for this
project to use.

With `wrangler`, you can build, test, and deploy your Worker with the following commands:

```bash
# you need to log into your account, not sure how long this persists
wrangler login

# compiles your project to WebAssembly and will warn of any issues
wrangler build 

# test your Worker locally
wrangler dev

./perform.sh load
./perform.sh put '{"id":4, "title":"title1", "body":"loren ipsum"}'
./perform.sh get 4

# If you get an error `KV_FROM_RUST is not defined` then you need to login

# deploy your Worker globally to the Cloudflare network (update your wrangler.toml file for configuration)
wrangler publish
```

Read the latest `worker` crate documentation here: `https://docs.rs/worker`

## WebAssembly

`workers-rs` (the Rust SDK for Cloudflare Workers used in this template) is meant to be executed as
compiled WebAssembly, and as such so **must** all the code you write and depend upon. All crates and
modules used in Rust-based Workers projects have to compile to the `wasm32-unknown-unknown` triple.

Read more about this on the [`workers-rs` project README](https://github.com/cloudflare/workers-rs).

## Troubleshooting Issues

If you have any problems with the `worker` crate, please open an issue on the upstream project
issue tracker on the [`workers-rs` repository](https://github.com/cloudflare/workers-rs).

### `ReferenceError: KV_FROM_RUST is not defined`

Your session has expired and you need to `wrangler login`

## Production

`https://workers-kv-from-rust.joemooney.workers.dev`
