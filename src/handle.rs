extern crate wasm_bindgen;
use crate::kvapi::KvAPI;
use crate::kvdb::KvDB;
use crate::kv::KeyValue;
use web_sys::{Request, Response, ResponseInit};
use wasm_bindgen::{prelude::*, JsCast};
use crate::utils::{return_ok, return_error, info};


#[wasm_bindgen]
/// Called by ../worker/worker.js
/// This is the inbound bridge from JS and Rust
/// Calls [`handle_json`]: #function.handle_json
/// kv_api: this is the cloudflare object representing the KV database passed from JavaScript
pub async fn handle(kv_api: KvAPI, req: JsValue) -> Result<Response, JsValue> {
    // rust wrapper for javascript api
    let kvdb = KvDB::new(kv_api);

    let req: Request = req.dyn_into()?;

    // from URL we get the key and value from the body
    let url = web_sys::Url::new(&req.url())?;

    // we use pathname /kv/1 as the key
    let pathname = url.pathname();

    info!("Url pathname:{}", pathname);
    let query_params = url.search_params();

    // extract if this is application/json or text
    let content_type = req.headers().get("Content-Type").unwrap_or_default().unwrap_or_default().to_lowercase();
    let accept = req.headers().get("Accept").unwrap_or_default().unwrap_or_default().to_lowercase();
    info!("Content-Type:{}", content_type);
    info!("Accept:{}", accept);

    // extract if this is GET POST PUT etc.
    let http_method = req.method().to_owned();

    match http_method.as_str() {
        "GET" => {
            if content_type == "application/json" {
                // the query is in JSON format
                return_ok(None)
            } else {
                let value = kvdb.get_key(&pathname).await?.unwrap_or_default();
                return_ok(Some(&value))
            }
        }
        "PUT" | "POST" => {
            // we were given some JSON (note it is not essential for content_type to be this)
            if content_type == "application/json" {
                handle_json(req, &http_method, kvdb).await
            } else {
                info!("POST no json {}", content_type);
                let value = query_params.get("value").unwrap_or_default();
                info!("POST store {}:{}", pathname, value);
                kvdb.put_text(&pathname, &value, 600).await?;
                return_ok(None)
            }
        }
        _ => {
            let mut init = ResponseInit::new();
            init.status(400);
            Response::new_with_opt_str_and_init(None, &init)
        }
    }
}


/// Received a JSON object
async fn handle_json(req: Request, _req_type: &str, kvdb: KvDB) -> Result<Response, JsValue> {
    let json_promise = req.json();
    // try to see if it is json
    info!("handle_json");
    if let Ok(p) = json_promise {
        match wasm_bindgen_futures::JsFuture::from(p).await {
            Ok(val) =>  {
                info!("handle_json into_serde");
                if let Ok(kv) = val.into_serde::<KeyValue>() {
                // } else if let Ok(kv) = serde_wasm_bindgen::from_value::<KeyValue>(val.to_owned()) {
                    kvdb.store_kv(kv).await
                } else {
                    return_error!("PUT/POST JSON object failed to parse: {:?}", val);
                }
            }
            Err(e) => {
                return_error!("[error] POST JSON from client is invalid: {:?}", e);
            }
        }
    } else {
        return_error!("POST invalid JSON: {:?}", req);
    }
}
