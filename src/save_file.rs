use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn saveFile(data: &[u8], filename: &str);
}

#[wasm_bindgen]
pub fn save_file_from_rust(data: Vec<u8>) {
    let filename = "output.pdf";

    // Convert Rust data to a byte slice
    let data_ptr = data.as_ptr();
    let data_len = data.len();

    // Safety: This is safe as long as `data` is not modified after this call.
    unsafe {
        saveFile(std::slice::from_raw_parts(data_ptr, data_len), filename);
    }
}