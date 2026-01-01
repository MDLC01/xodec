use std::{collections::HashMap, path::Path};

use codex::{Def, Module};

fn explore_module(
    symbols: &mut Vec<(String, &'static str)>,
    deprecated: &mut Vec<(String, &'static str)>,
    m: Module,
    name: &str,
    is_deprecated: bool,
) {
    for (id, binding) in m.iter() {
        let new_name = if name.is_empty() {
            id.to_owned()
        } else {
            format!("{name}.{id}")
        };
        let binding_is_deprecated = is_deprecated || binding.deprecation.is_some();
        match binding.def {
            Def::Symbol(s) => {
                for (modifiers, value, deprecation) in s.variants() {
                    let mut full_name = new_name.clone();
                    if !modifiers.is_empty() {
                        full_name.push('.');
                        full_name.push_str(modifiers.as_str())
                    }
                    let variant_is_deprecated = binding_is_deprecated || deprecation.is_some();
                    if variant_is_deprecated {
                        deprecated.push((full_name, value))
                    } else {
                        symbols.push((full_name, value))
                    }
                }
            }
            Def::Module(m2) => {
                explore_module(symbols, deprecated, m2, &new_name, binding_is_deprecated)
            }
        }
    }
}

fn encode_symbol_list(buf: &mut String, name: &str, symbols: Vec<(String, &'static str)>) {
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

    buf.push_str(&format!(
        "static {}: [(&[u8], &[u8]); {}] = [\n",
        name,
        encoded_symbol_list.len()
    ));
    for (value, names) in encoded_symbol_list {
        buf.push_str(&format!("    (&{value:?}, &{names:?}),\n"));
    }
    buf.push_str("];\n");
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let mut symbols = Vec::new();
    let mut deprecated = Vec::new();
    explore_module(&mut symbols, &mut deprecated, codex::ROOT, "", false);

    let mut buf = String::new();
    encode_symbol_list(&mut buf, "SYMBOLS", symbols);
    encode_symbol_list(&mut buf, "DEPRECATED", deprecated);

    let out = std::env::var_os("OUT_DIR").unwrap();
    let dest = Path::new(&out).join("out.rs");
    std::fs::write(&dest, buf).unwrap();
}
