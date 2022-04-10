use wasm_bindgen::{prelude::*};
use web_sys::{Response, ResponseInit};

pub use js_console::log as console_log;
pub use js_console::log as console_debug;
pub use js_console::log as console_warn;
pub use js_console::log as console_error;

use cfg_if::cfg_if;

#[allow(unused)]
/// Print error to javascript console and return the error String so we can pass back to user in Response
macro_rules! error{ ($($args:tt)*) => 
    { 
        let s = format!($($args)*);
        utils::console_error(&s);
        return utils::return_status(400, &s);
    } 
}
#[allow(unused)]
pub(crate) use error;

#[allow(unused)]
macro_rules! debug{ ($($args:tt)*) => { let s = format!($($args)*); utils::console_debug(&s);  } }
#[allow(unused)]
pub(crate) use debug;

#[allow(unused)]
macro_rules! warning{ ($($args:tt)*) => { let s = format!($($args)*); utils::console_warn(&s);  } }
#[allow(unused)]
pub(crate) use warning;

macro_rules! info{ ($($args:tt)*) => { let s = format!($($args)*); utils::console_log(&s);  } }
pub(crate) use info;

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

pub fn return_ok() -> Result<Response, JsValue> {
    // set a TTL of 600 seconds:
    let mut init = ResponseInit::new();
    init.status(200);
    Response::new_with_opt_str_and_init(None, &init)
}

pub fn return_status(status: u16, msg: &str) -> Result<Response, JsValue> {
    // set a TTL of 600 seconds:
    let mut init = ResponseInit::new();
    init.status(status);
    Response::new_with_opt_str_and_init(Some(msg), &init)
}

// First up let's take a look of binding `console.log` manually, without the
// help of `web_sys`. Here we're writing the `#[wasm_bindgen]` annotations
// manually ourselves, and the correctness of our program relies on the
// correctness of these annotations!

mod js_console {
    use wasm_bindgen::prelude::*;

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
}