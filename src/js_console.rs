use wasm_bindgen::{prelude::*};
// use web_sys::{Response, ResponseInit};


// First up let's take a look of binding `console.log` manually, without the
// help of `web_sys`. Here we're writing the `#[wasm_bindgen]` annotations
// manually ourselves, and the correctness of our program relies on the
// correctness of these annotations!

// use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.debug(..)` instead of just
    // `debug(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn debug(s: &str);

    // Use `js_namespace` here to bind `console.warn(..)` instead of just
    // `warn(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(s: &str);

    // Use `js_namespace` here to bind `console.error(..)` instead of just
    // `error(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_many(a: &str, b: &str);
}