use std::{collections::HashMap, sync::Mutex};

extism_convert::encoding!(pub Postcard, postcard::to_allocvec, postcard::from_bytes);

#[cfg(not(target_arch = "wasm32"))]
lazy_static::lazy_static! {
    pub static ref WASM_INSTANCE: Mutex<HashMap<String, extism::Plugin>> = {
        Mutex::new(HashMap::new())
    };
}
