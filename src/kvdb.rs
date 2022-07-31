use crate::kvapi::KvAPI;
use wasm_bindgen::{prelude::*};
use js_sys::{ArrayBuffer, Object, Reflect, Uint8Array};
// use crate::js_console::*;
use js_sys::{Date};
use crate::kv::KeyValue;
use crate::utils::{info, get_expiration, return_error, return_ok};
use web_sys::{Response};

/// Wrapper for Cloudflare Worker KV database connection
pub struct KvDB {
    /// JavaScript handle/connection to the KV Database
    /// The KV API is documented here:
    /// https://developers.cloudflare.com/workers/runtime-apis/kv/
    kv_api: KvAPI,
}

impl KvDB {
    pub fn new(kv_api: KvAPI) -> Self {
        Self { kv_api }
    }
    #[allow(dead_code)]
    async fn put_jsvalue(&self, key: &str, value: JsValue, expiration_ttl: u64) -> Result<(), JsValue> {
        println!("put object with expiration_ttl:{}", expiration_ttl);
        info!("putting object with expiration_ttl {}, key:{}", expiration_ttl, key);
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        self.kv_api
            .put(JsValue::from_str(key), value, options.into())
            .await?;
        info!("put object with expiration_ttl {}", expiration_ttl);
        Ok(())
    }

    pub async fn put_text(&self, key: &str, value: &str, expiration_ttl: u64) -> Result<(), JsValue> {
        //println!("put object with expiration_ttl:{}", expiration_ttl);
        info!(">put key:{} value:{} expiration:{}", key, value, expiration_ttl);
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        self.kv_api
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
        // expirationTtl is number of seconds in Unix Time since epoch
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        let typed_array = Uint8Array::new_with_length(value.len() as u32);
        typed_array.copy_from(value);
        self.kv_api
            .put(
                JsValue::from_str(key),
                typed_array.buffer().into(),
                options.into(),
            )
            .await?;
        Ok(())
    }

    /// Cloudflare storage option: "text", "json", "arrayBuffer"
    fn storage_format(format: &str) -> Result<Object, JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"type".into(), &format.into())?;
        Ok(options)
    }

    pub async fn get_key(&self, key: &str) -> Result<Option<String>, JsValue> {
        let options = KvDB::storage_format("text")?;
        // let options = Object::new();
        // Reflect::set(&options, &"type".into(), &"text".into())?;
        info!(">get key:{}", key);
        let value = self
            .kv_api
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
        let value = self.kv_api.get(JsValue::from_str(key), options.into()).await?;
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

    pub async fn store_kv(&self, kv: KeyValue) -> Result<Response, JsValue> {
        info!("handle_json into_serde got KeyValue");
        info!("handle_json into_serde got KeyValue into string");
    
        // extract the expiration from the KeyValue, we will also
        // store the expiration in the KeyValue
        let expiration = get_expiration!(kv);
    
        let system_time = Date::now();
        // let zero = std::time::Timespec { sec: 0, nsec: 0};
        // let system_time = time::Instant::now();
        info!("handle_json into_serde got KeyValue got time {:?}", system_time);
        // let _datetime: DateTime<Utc> = system_time.into();
        info!("handle_json into_serde got KeyValue converted time");
    
        let value = match serde_json::to_string(&kv.value) {
            Ok(value) => value,
            Err(e) => {return_error!("POST json KeyValue serialization error: {:?}", e);}
        };
    
        info!("store kv {}", &value);
        // store KeyValue
        self.put_text(&format!("/kv/{}", kv.key), &value, expiration).await?;
        return_ok()
    }

}