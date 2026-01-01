use wasm_minimal_protocol::{initiate_protocol, wasm_func};

initiate_protocol!();

include!(concat!(env!("OUT_DIR"), "/out.rs"));

#[wasm_func]
pub fn get_names(symbol: &[u8]) -> Vec<u8> {
    match SYMBOLS.binary_search_by_key(&symbol, |(value, _)| value) {
        Err(_) => Vec::new(),
        Ok(i) => SYMBOLS[i].1.to_owned(),
    }
}
