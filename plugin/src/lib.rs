use wasm_minimal_protocol::{initiate_protocol, wasm_func};

initiate_protocol!();

include!(concat!(env!("OUT_DIR"), "/out.rs"));

#[wasm_func]
pub fn get_names(symbol: &[u8]) -> Vec<u8> {
    find_sym_names(symbol).to_owned()
}

#[wasm_func]
pub fn get_deprecated_names(symbol: &[u8]) -> Vec<u8> {
    find_deprecated_names(symbol).to_owned()
}

#[wasm_func]
pub fn get_math_names(symbol: &[u8]) -> Vec<u8> {
    find_math_names(symbol).to_owned()
}
