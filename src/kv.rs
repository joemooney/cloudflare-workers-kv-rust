// use wasm_bindgen::{prelude::*};
use serde::{Serialize, Deserialize};
// use crate::duration_helper::DurationHelper;
// use crate::utils::{self, info, return_error};


/// KeyValue is a single String blob (typically JSON)
/// that we wish to persist
/// stored with a unique id, title, and expiration
/// The expiration defaults to 24 hours.
#[derive(Serialize, Deserialize)]
pub struct KeyValue {
    pub key: u32,
    pub value: String,
    pub metadata: Option<MetaData>,
}

/// MetaData associated with a KeyValue
/// Expiration, Tags, etc. are stored in metadata
#[derive(Serialize, Deserialize)]
pub struct MetaData {
    /// format of data in body, e.g. JSON
    // #[serde(default = "JSON")] 
    pub format: String,

    /// comma separated list of tags
    pub tags: String,

    /// 1h, 1hr, 1d, 1day, 1yr, 1month etc
    /// Expiration Date e.g 20220805.191107
    // #[serde(default = "1 day")] 
    pub expiration: String,

    /// Date created e.g 20220805.191107
    pub created: String,
}

