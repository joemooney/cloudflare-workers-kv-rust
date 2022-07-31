extern crate cfg_if;
extern crate wasm_bindgen;
// use std::sync::Arc;
// use std::time::UNIX_EPOCH;
//use instant;

// use serde::{Serialize, Deserialize};

mod utils;
mod kvapi;
mod kvdb;
mod kv;
mod duration_helper;
mod js_console;

use crate::kvapi::KvAPI;
use crate::kvdb::KvDB;
use crate::kv::KeyValue;

pub use js_console::*;

// use duration_helper::DurationHelper;

use cfg_if::cfg_if;
// use js_sys::{Date, ArrayBuffer, Object, Reflect, Uint8Array};
// use js_sys::{Date};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Request, Response, ResponseInit};

enum RequestType {
    PUT,
    POST,
}

// #[cfg(not(target_arch = "wasm32"))]
// use log::{error, info};

// #[cfg(target_arch = "wasm32")]
use utils::{return_error, info};

// use chrono::offset::Utc;
// use chrono::{Duration, DateTime};

//use std::time::SystemTime;
// SystemTime::now() is crashing
// using this instead `https://github.com/rust-lang/rust/issues/48564`

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// #[derive(Serialize, Deserialize)]
// pub struct Db {
//     pub data: Vec<KeyValue>,
// }



// given 1day return number of seconds
// 1d1h -> 25 hours in seconds
// 2days === 2day ===2d 
// 2seconds === 2sec ===2s 

// fn day() -> String {
//     "1day".to_owned()
// }

#[wasm_bindgen]
/// Calls [`handle_json`]: #function.handle_json
/// Called by ../worker/worker.js
/// kv_api: this is the cloudflare object representing the KV database passed from JavaScript
pub async fn handle(kv_api: KvAPI, req: JsValue) -> Result<Response, JsValue> {
    let kvdb = KvDB::new(kv_api); // rust wrapper for javascript api
    let req: Request = req.dyn_into()?;
    let url = web_sys::Url::new(&req.url())?;
    let pathname = url.pathname();
    info!("Url pathname:{}", pathname);
    let query_params = url.search_params();
    let content_type = req.headers().get("Content-Type").unwrap_or_default().unwrap_or_default().to_lowercase();
    let http_method = req.method().to_owned();
    match http_method.as_str() {
        "GET" => {
            if content_type == "application/json" {
                // the query is in JSON format
                utils::return_ok()
            } else {
                let value = kvdb.get_key(&pathname).await?.unwrap_or_default();
                let mut init = ResponseInit::new();
                init.status(200);
                Response::new_with_opt_str_and_init(Some(&format!("\"{}\"\n", value)), &init)
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
                utils::return_ok()
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
async fn handle_json(req: Request, req_type: &str, kvdb: KvDB) -> Result<Response, JsValue> {
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


