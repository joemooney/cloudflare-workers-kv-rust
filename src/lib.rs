extern crate cfg_if;
extern crate wasm_bindgen;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
//use instant;

use serde::{Serialize, Deserialize};

mod utils;

use cfg_if::cfg_if;
use js_sys::{Date, ArrayBuffer, Object, Reflect, Uint8Array};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Request, Response, ResponseInit};

use utils::{error, info};
use chrono::offset::Utc;
use chrono::{Duration, DateTime};

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
//     pub data: Vec<Datum>,
// }

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub product_id: u32,
    pub quantity: u32,
}
#[derive(Serialize, Deserialize)]
pub struct Datum {
    pub id: u32,
    pub title: String,
    pub body: String,
    #[serde(default = "day")] 
    pub expiration: String,
}

enum TimeUnit {
    Minute,
    Hour,
    Second,
    Day,
    Week,
    Month,
    Year,
}

struct TimeParse {
    spec: String,
    duration: Duration,
    current_unit: String,
    unit: TimeUnit,
    count: i64,
    total: i64,
}
impl TimeParse {
    pub fn tally(&mut self) -> Result<(), String>{
        info!(">tally current_unit:{} count:{} duration:{:?}", self.current_unit, self.count, self.duration);
        if self.current_unit == "" {
            return Ok(())
        }
        self.check_unit()?;
        if self.count != 0 {
            let d = match self.unit {
                TimeUnit::Second => self.duration.checked_add(&Duration::seconds(self.count)),
                TimeUnit::Minute => self.duration.checked_add(&Duration::minutes(self.count)),
                TimeUnit::Hour => self.duration.checked_add(&Duration::hours(self.count)),
                TimeUnit::Day => self.duration.checked_add(&Duration::days(self.count)),
                TimeUnit::Week => self.duration.checked_add(&Duration::weeks(self.count)),
                TimeUnit::Month => self.duration.checked_add(&Duration::days(self.count*30)),
                TimeUnit::Year => self.duration.checked_add(&Duration::weeks(self.count*52)),
            };
            match d {
                Some(d) => self.duration = d,
                None => return Err(format!("Invalid duration {}", self.spec))
            }
        }
        info!("<tally current_unit:{} count:{} duration:{:?}", self.current_unit, self.count, self.duration);
        self.current_unit = "".to_owned();
        self.count = 0;
        Ok(())
    }

    fn invalid(&self) -> Result<(), String> {
        return Err(format!("Invalid duration spec:{}", self.spec))
    }

    fn check_unit(&mut self) -> Result<(), String> {
        info!("check_unit: <{}>", self.current_unit);
        if self.current_unit == "" {
            if self.count > 0 {
                self.invalid()?;
            }
        }
        self.unit = match self.current_unit.to_lowercase().as_str() {
            "h" | "hr" | "hrs" | "hours"  | "hour" => TimeUnit::Hour,
            "m" | "min"| "mins" | "minute" | "minutes" => TimeUnit::Minute,
            "s" | "sec"| "secs" | "second" | "seconds" => TimeUnit::Second,
            "d" | "day"| "days"  => TimeUnit::Day,
            "mon"| "months"  => TimeUnit::Month,
            "w" | "wk"| "wks" | "week" | "weeks"  => TimeUnit::Week,
            "y" | "yr"| "yrs" | "year" | "years"  => TimeUnit::Year,
            _ => return self.invalid(),
        };
        Ok(())
    }
    fn convert(duration: &str) -> Result<i64, String> {
        let mut t = TimeParse{ duration: Duration::zero(), spec: duration.to_owned(), total: 0, count: 0, unit: TimeUnit::Second, current_unit: "".to_owned() };
        t.calc()?;
        Ok(t.duration.num_seconds())
    }
    fn calc(&mut self) -> Result<i64, String> {
        let chars = self.spec.chars().collect::<Vec<char>>();
        info!("parse: {:?}", chars);
        for c in chars {
            info!("char: {}", c);
            if c.is_whitespace() {
                self.tally()?;
            } else if c == '-' {
                self.tally()?;
                self.count = -1;
            } else if c.is_digit(10) {
                if self.current_unit != "" {
                    self.tally()?;
                }
                let d = (c.to_string()).parse::<i64>().unwrap(); 
                self.count *= 10;
                if self.count < 0 {
                    self.count -= d;
                } else {
                    self.count += d;
                }
            } else {
                if self.current_unit == "" {
                    self.tally()?;
                }
                self.current_unit.push(c)
            }
        }
        self.tally()?;
        Ok(self.count)
    }
}

/// given 1day return number of seconds
/// 1d1h -> 25 hours in seconds
/// 2days === 2day ===2d 
/// 2seconds === 2sec ===2s 

fn day() -> String {
    "1day".to_owned()
}

#[wasm_bindgen]
/// Calls [`handle_json`]: #function.handle_json
pub async fn handle(kv: WorkersKvJs, req: JsValue) -> Result<Response, JsValue> {
    let req: Request = req.dyn_into()?;
    let url = web_sys::Url::new(&req.url())?;
    let pathname = url.pathname();
    info!("Url pathname:{}", pathname);
    let query_params = url.search_params();
    let kv = WorkersKv { kv };
    let content_type = req.headers().get("Content-Type").unwrap_or_default().unwrap_or_default().to_lowercase();
    match req.method().as_str() {
        "GET" => {
            if content_type == "application/json" {
                // the query is in JSON format
                utils::return_ok()
            } else {
                let value = kv.get_text(&pathname).await?.unwrap_or_default();
                let mut init = ResponseInit::new();
                init.status(200);
                Response::new_with_opt_str_and_init(Some(&format!("\"{}\"\n", value)), &init)
            }
        }
        // "PUT" => {
        //     let value = query_params.get("value").unwrap_or_default();
        //     let json_promise = req.json();
        //     if let Ok(p) = json_promise {
        //         info!(&format!("put object {:?}", p));
        //     }
        //     // set a TTL of 600 seconds:
        //     kv.put_text(&pathname, &value, 600).await?;
        //     let mut init = ResponseInit::new();
        //     init.status(200);
        //     Response::new_with_opt_str_and_init(None, &init)
        // }
        "PUT" | "POST" => {
            // we were given some JSON (note it is not essential for content_type to be this)
            if content_type == "application/json" {
                handle_json(req, kv).await
            } else {
                info!("POST no json {}", content_type);
                let value = query_params.get("value").unwrap_or_default();
                info!("POST store {}:{}", pathname, value);
                kv.put_text(&pathname, &value, 600).await?;
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
    #[allow(dead_code)]
    async fn put_jsvalue(&self, key: &str, value: JsValue, expiration_ttl: u64) -> Result<(), JsValue> {
        println!("put object with expiration_ttl:{}", expiration_ttl);
        info!("putting object with expiration_ttl {}, key:{}", expiration_ttl, key);
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        self.kv
            .put(JsValue::from_str(key), value, options.into())
            .await?;
        info!("put object with expiration_ttl {}", expiration_ttl);
        Ok(())
    }

    async fn put_text(&self, key: &str, value: &str, expiration_ttl: u64) -> Result<(), JsValue> {
        //println!("put object with expiration_ttl:{}", expiration_ttl);
        info!(">put key:{} value:{} expiration:{}", key, value, expiration_ttl);
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        self.kv
            .put(JsValue::from_str(key), value.into(), options.into())
            .await?;
        // info!("<put object with expiration_ttl {}", expiration_ttl);
        info!("<put key:{} value:{} expiration:{}", key, value, expiration_ttl);
        Ok(())
    }

    #[allow(dead_code)]
    async fn put_vec(&self, key: &str, value: &[u8], expiration_ttl: u64) -> Result<(), JsValue> {
        let options = Object::new();
        info!("put vec object with expiration_ttl");
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
        info!(">get key:{}", key);
        let value = self
            .kv
            .get(JsValue::from_str(key), options.into())
            .await?
            .as_string();
        if let Some(v) = &value {
            info!("<get key:{} value:{}", key, v);
        } else {
            info!("<get key:{} NOT_FOUND", key);
        }
        Ok(value)
    }

    #[allow(dead_code)]
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

/// Received a JSON object
async fn handle_json(req: Request, kv: WorkersKv) -> Result<Response, JsValue> {
    let json_promise = req.json();
    // try to see if it is json
    info!("handle_json");
    if let Ok(p) = json_promise {
        match wasm_bindgen_futures::JsFuture::from(p).await {
            Ok(val) =>  {
                info!("handle_json into_serde");
                if let Ok(product) = val.into_serde::<Product>() {
                //if let Ok(product) = serde_wasm_bindgen::from_value::<Product>(val.to_owned()) {
                    info!("got json product object {}", product.product_id);
                    if let Ok(value) = serde_json::to_string(&product) {
                        info!("storing product object {}", value);
                        //kv.put_jsvalue(&format!("Product:{}", product.product_id), val, 600).await?;
                        kv.put_text(&format!("/Product:{}", product.product_id), &value, 600).await?;
                        utils::return_ok()
                    } else {
                        // internal errors should never happen
                        error!("json Product serialization error: {:?}", val);
                    }
                } else if let Ok(datum) = val.into_serde::<Datum>() {
                // } else if let Ok(datum) = serde_wasm_bindgen::from_value::<Datum>(val.to_owned()) {
                    handle_datum(datum, kv).await
                } else {
                    error!("PUT/POST JSON object failed to parse: {:?}", val);
                }
            }
            Err(e) => {
                error!("[error] POST JSON from client is invalid: {:?}", e);
            }
        }
    } else {
        error!("POST invalid JSON: {:?}", req);
    }
}

async fn handle_datum(datum: Datum, kv: WorkersKv) -> Result<Response, JsValue> {
    info!("handle_json into_serde got Datum");
    info!("handle_json into_serde got Datum into string");
    let expiration = match TimeParse::convert(&datum.expiration) {
        Ok(f) => {
            info!("handle_json into_serde got Datum into expiration {}={}", datum.expiration, f);
            f as u64
        }
        Err(e) => {
            error!("invalid expiration: {}", e);
        }
    };
    let system_time = Date::now();
    // let zero = std::time::Timespec { sec: 0, nsec: 0};
    // let system_time = time::Instant::now();
    info!("handle_json into_serde got Datum got time {:?}", system_time);
    // let _datetime: DateTime<Utc> = system_time.into();
    info!("handle_json into_serde got Datum converted time");

    let value = match serde_json::to_string(&datum) {
        Ok(value) => value,
        Err(e) => {error!("POST json Datum serialization error: {:?}", e);}
    };

    info!("store datum {}", &value);
    kv.put_text(&format!("/datum/{}", datum.id), &value, expiration).await?;
    utils::return_ok()
}
