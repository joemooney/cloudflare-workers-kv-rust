use wasm_bindgen::{prelude::*};
use web_sys::{Response, ResponseInit};
use cfg_if::cfg_if;

// use super::js_console;



cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn return_ok(body: Option<&str>) -> Result<Response, JsValue> {
    let mut init = ResponseInit::new();
    init.status(200);
    Response::new_with_opt_str_and_init(body, &init)
}

pub fn return_status(status: u16, msg: &str) -> Result<Response, JsValue> {
    let mut init = ResponseInit::new();
    init.status(status);
    Response::new_with_opt_str_and_init(Some(msg), &init)
}

// #[cfg(target_arch = "wasm32")]
#[allow(unused)]
/// Print error to javascript console and return the error String so we can pass back to user in Response
macro_rules! return_error{ ($($args:tt)*) => 
    { 
        let s = format!($($args)*);
        crate::js_console::error(&s);
        return crate::utils::return_status(400, &s);
    } 
}
// #[cfg(target_arch = "wasm32")]
#[allow(unused)]
pub(crate) use return_error;

// #[cfg(target_arch = "wasm32")]
#[allow(unused)]
macro_rules! debug{ ($($args:tt)*) => 
    { 
        let s = format!($($args)*); 
        crate::js_console::debug(&s);  
    } }
// #[cfg(target_arch = "wasm32")]
#[allow(unused)]
pub(crate) use debug;

// #[cfg(target_arch = "wasm32")]
#[allow(unused)]
macro_rules! warning{ ($($args:tt)*) => 
    { 
        let s = format!($($args)*); 
        crate::js_console::warn(&s);  
    } }

// #[cfg(target_arch = "wasm32")]
#[allow(unused)]
pub(crate) use warning;

// #[cfg(target_arch = "wasm32")]
#[allow(unused)]
macro_rules! info{ ($($args:tt)*) => { 
    let s = format!($($args)*); 
    crate::js_console::log(&s);  
    } }
// #[cfg(target_arch = "wasm32")]
#[allow(unused)]
pub(crate) use info;


macro_rules! get_expiration{ ($kv:expr) => 
    { 
        if let Some(metadata) = &$kv.metadata {
            match crate::duration_helper::DurationHelper::convert(&metadata.expiration) {
                Ok(f) => {
                    info!("handle_json into_serde got KeyValue into expiration {}={}", metadata.expiration, f);
                    f as u64
                }
                Err(e) => {
                    return_error!("invalid expiration: {}", e);
                }
            }
        } else {
            24*60*60 // one day default expiration
        }
    } 
}
#[allow(unused)]
pub(crate) use get_expiration;