use wasm_minimal_protocol::{initiate_protocol, wasm_func};

initiate_protocol!();

fn find_names<'a>(symbol: &[u8], list: &[(&[u8], &'a [u8])]) -> &'a [u8] {
    match list.binary_search_by_key(&symbol, |(value, _)| value) {
        Err(_) => &[],
        Ok(i) => list[i].1,
    }
}

include!(concat!(env!("OUT_DIR"), "/out.rs"));

#[wasm_func]
pub fn get_names(symbol: &[u8]) -> Vec<u8> {
    find_names(symbol, &SYMBOLS).to_owned()
}

#[wasm_func]
pub fn get_deprecated_names(symbol: &[u8]) -> Vec<u8> {
    find_names(symbol, &DEPRECATED).to_owned()
}
