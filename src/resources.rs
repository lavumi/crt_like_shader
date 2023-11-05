use cfg_if::cfg_if;
use toml::Value;
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;
// #[cfg(target_arch = "wasm32")]
// use js_sys::{Promise, Uint8Array};
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen_futures::JsFuture;
//
//
// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_name = loadImage)]
//     fn js_load_image(url: &str) -> Promise;
// }
//
//
// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen]
// pub async fn load_image(url: &str) -> Result<bool, JsValue> {
//     let promise = js_load_image(url);
//     let array_buffer = JsFuture::from(promise).await?;
//     let uint8_array = Uint8Array::new(&array_buffer);
//
//     // Uint8Array를 Vec<u8>로 변환
//     let mut result = Vec::new();
//     for i in 0..uint8_array.length() {
//         result.push(uint8_array.get_index(i) as u8);
//     }
//
//     Ok(true)
// }


#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

#[allow(unused)]
pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

// pub async fn load_toml(file_name: &str) -> anyhow::Result<Value> {
//     let txt = load_string(file_name).await?;
//     let toml_value: Value = toml::de::from_str(&txt)
//         .expect("Failed to parse TOML");
//
//     Ok(toml_value)
// }