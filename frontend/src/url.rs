//! URL helpers. Leptos' `ParamsMap` keeps only one value per key, so repeated
//! params (`?q=cat&q=dog`) are read straight from `location.search`.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function queryAll(key) {
  return new URLSearchParams(window.location.search).getAll(key);
}

export function encodeComponent(s) {
  return encodeURIComponent(s);
}
"#)]
extern "C" {
    /// Every value of `key` in the current page's query string.
    #[wasm_bindgen(js_name = queryAll)]
    pub fn query_all(key: &str) -> Vec<String>;

    /// `encodeURIComponent`.
    #[wasm_bindgen(js_name = encodeComponent)]
    pub fn encode_component(s: &str) -> String;
}
