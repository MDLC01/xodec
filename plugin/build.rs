use std::{collections::HashMap, path::Path};

use codex::{Def, Module};

fn explore_module(vec: &mut Vec<(String, &'static str)>, m: Module, name: &str) {
    for (id, binding) in m.iter() {
        let new_name = if name.len() == 0 {
            id.to_owned()
        } else {
            format!("{name}.{id}")
        };
        match binding.def {
            Def::Symbol(s) => {
                for (modifiers, value, _) in s.variants() {
                    let mut full_name = new_name.clone();
                    if !modifiers.is_empty() {
                        full_name.push('.');
                        full_name.push_str(modifiers.as_str())
                    }
                    vec.push((full_name, value))
                }
            }
            Def::Module(m2) => explore_module(vec, m2, &new_name),
        }
    }
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let mut symbols = Vec::new();
    explore_module(&mut symbols, codex::ROOT, "");
    let mut symbols_by_value = HashMap::<&'static str, Vec<String>>::new();
    for (name, value) in symbols {
        symbols_by_value.entry(value).or_default().push(name);
    }
    let mut encoded_symbol_list = symbols_by_value
        .into_iter()
        .map(|(value, names)| {
            let mut encoded_names = Vec::new();
            for name in names {
                encoded_names.extend_from_slice(name.as_bytes());
                encoded_names.push(0);
            }
            encoded_names.pop();
            (value.as_bytes(), encoded_names)
        })
        .collect::<Vec<_>>();
    encoded_symbol_list.sort_by_key(|(value, _)| *value);

    let mut buf = format!(
        "static SYMBOLS: [(&[u8], &[u8]); {}] = [\n",
        encoded_symbol_list.len(),
    );
    for (value, names) in encoded_symbol_list {
        buf.push_str(&format!("    (&{value:?}, &{names:?}),\n"));
    }
    buf.push_str("];\n");

    let out = std::env::var_os("OUT_DIR").unwrap();
    let dest = Path::new(&out).join("out.rs");
    std::fs::write(&dest, buf).unwrap();
}
