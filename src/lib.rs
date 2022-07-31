extern crate cfg_if;
extern crate wasm_bindgen;

mod handle;
mod utils;
mod kvapi;
mod kvdb;
mod kv;
mod duration_helper;
mod js_console;

/// expose the handle function to worker/worker.js javascript
/// this is the handler for all incoming requests
pub use handle::handle;


// Optimization:
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}