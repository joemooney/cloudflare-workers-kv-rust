extern crate cfg_if;
extern crate wasm_bindgen;
use serde::{Serialize, Deserialize};

mod utils;

use cfg_if::cfg_if;
use js_sys::{ArrayBuffer, Object, Reflect, Uint8Array};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Request, Response, ResponseInit};

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub product_id: u32,
    pub quantity: u32,
}

#[wasm_bindgen]
pub async fn handle(kv: WorkersKvJs, req: JsValue) -> Result<Response, JsValue> {
    let req: Request = req.dyn_into()?;
    let url = web_sys::Url::new(&req.url())?;
    let pathname = url.pathname();
    let query_params = url.search_params();
    let kv = WorkersKv { kv };
    let headers = req.headers();
    let content_type = req.headers().get("Content-Type").unwrap_or_default().unwrap_or_default().to_lowercase();
    match req.method().as_str() {
        "GET" => {
            let value = kv.get_text(&pathname).await?.unwrap_or_default();
            let mut init = ResponseInit::new();
            init.status(200);
            Response::new_with_opt_str_and_init(Some(&format!("\"{}\"\n", value)), &init)
        }
        "PUT" => {
            let value = query_params.get("value").unwrap_or_default();
            let json_promise = req.json();
            if let Ok(p) = json_promise {
                utils::log(&format!("put object {:?}", p));
            }
            // set a TTL of 600 seconds:
            kv.put_text(&pathname, &value, 600).await?;
            let mut init = ResponseInit::new();
            init.status(200);
            Response::new_with_opt_str_and_init(None, &init)
        }
        "POST" => {
            if content_type == "application/json" {
                let json_promise = req.json();
                utils::log(&format!("7 - handle request"));
                // try to see if it is json
                if let Ok(p) = json_promise {
                    match wasm_bindgen_futures::JsFuture::from(p).await {
                        Ok(val) =>  {
                            if let Ok(product) = val.into_serde::<Product>() {
                                utils::log(&format!("got json product object {}", product.product_id));
                                if let Ok(value) = serde_json::to_string(&product) {
                                    utils::log(&format!("storing product object {}", product.product_id));
                                    kv.put_text(&format!("Product:{}", product.product_id), &value, 600).await?;
                                }
                            } else {
                                utils::log(&format!("post json object {:?}", val));
                                utils::log(&format!("post json object failed to parse Product"));
                            }
                        }
                        Err(e) => {
                            utils::log(&format!("POST JSON invalid: {:?}", e))
                        }
                    }
                } else {
                    utils::log(&format!("POST invalid JSON"));
                }
            } else {
                utils::log(&format!("POST no json {}", content_type));
                let value = query_params.get("value").unwrap_or_default();
                kv.put_text(&pathname, &value, 600).await?;
            }
            // set a TTL of 600 seconds:
            let mut init = ResponseInit::new();
            init.status(200);
            Response::new_with_opt_str_and_init(None, &init)
        }
        _ => {
            let mut init = ResponseInit::new();
            init.status(400);
            Response::new_with_opt_str_and_init(None, &init)
        }
    }
}

#[wasm_bindgen]
extern "C" {
    pub type WorkersKvJs;

    #[wasm_bindgen(structural, method, catch)]
    pub async fn put(
        this: &WorkersKvJs,
        k: JsValue,
        v: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(structural, method, catch)]
    pub async fn get(
        this: &WorkersKvJs,
        key: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;
}

struct WorkersKv {
    kv: WorkersKvJs,
}

impl WorkersKv {
    async fn put_text(&self, key: &str, value: &str, expiration_ttl: u64) -> Result<(), JsValue> {
        println!("put object with expiration_ttl:{}", expiration_ttl);
        utils::log(&format!("putting object with expiration_ttl {}, key:{}", expiration_ttl, key));
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        self.kv
            .put(JsValue::from_str(key), value.into(), options.into())
            .await?;
        utils::log(&format!("put object with expiration_ttl {}", expiration_ttl));
        Ok(())
    }

    async fn put_vec(&self, key: &str, value: &[u8], expiration_ttl: u64) -> Result<(), JsValue> {
        let options = Object::new();
        utils::log("put vec object with expiration_ttl");
        println!("put vec object with expiration_ttl:{}", expiration_ttl);
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        let typed_array = Uint8Array::new_with_length(value.len() as u32);
        typed_array.copy_from(value);
        self.kv
            .put(
                JsValue::from_str(key),
                typed_array.buffer().into(),
                options.into(),
            )
            .await?;
        Ok(())
    }

    async fn get_text(&self, key: &str) -> Result<Option<String>, JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"type".into(), &"text".into())?;
        Ok(self
            .kv
            .get(JsValue::from_str(key), options.into())
            .await?
            .as_string())
    }

    async fn get_vec(&self, key: &str) -> Result<Option<Vec<u8>>, JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"type".into(), &"arrayBuffer".into())?;
        let value = self.kv.get(JsValue::from_str(key), options.into()).await?;
        if value.is_null() {
            Ok(None)
        } else {
            let buffer = ArrayBuffer::from(value);
            let typed_array = Uint8Array::new_with_byte_offset(&buffer, 0);
            let mut v = vec![0; typed_array.length() as usize];
            typed_array.copy_to(v.as_mut_slice());
            Ok(Some(v))
        }
    }
}