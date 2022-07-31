/// Functions that call the Cloudflare KV API
/// The KV API is documented here:
/// https://developers.cloudflare.com/workers/runtime-apis/kv/

use wasm_bindgen::{prelude::*};

#[wasm_bindgen]
extern "C" {
    pub type KvAPI;

    #[wasm_bindgen(structural, method, catch)]
    pub async fn put(
        this: &KvAPI,
        k: JsValue,
        v: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(structural, method, catch)]
    pub async fn get(
        this: &KvAPI,
        key: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;
}
