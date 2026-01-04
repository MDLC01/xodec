use codex::styling::{MathStyle, MathVariant, to_style};
use codex::{Def, Module};
use std::collections::HashSet;
use std::fmt::Display;
use std::{collections::HashMap, path::Path};
use typst_syntax::ast::MathShorthand;

fn explore_module(
    symbols: &mut Vec<(String, String)>,
    deprecated: &mut Vec<(String, String)>,
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
                        deprecated.push((full_name, value.to_owned()))
                    } else {
                        symbols.push((full_name, value.to_owned()))
                    }
                }
            }
            Def::Module(m2) => {
                explore_module(symbols, deprecated, m2, &new_name, binding_is_deprecated)
            }
        }
    }
}

#[derive(Copy, Clone)]
struct MathStyleConfig {
    variant: MathVariant,
    bold: bool,
    italic: Option<bool>,
}

impl MathStyleConfig {
    fn iter() -> impl Iterator<Item = MathStyleConfig> {
        [
            MathVariant::Plain,
            MathVariant::Fraktur,
            MathVariant::SansSerif,
            MathVariant::Monospace,
            MathVariant::DoubleStruck,
            MathVariant::Chancery,
            MathVariant::Roundhand,
        ]
        .into_iter()
        .flat_map(|variant| {
            [
                Self {
                    variant,
                    bold: false,
                    italic: None,
                },
                Self {
                    variant,
                    bold: false,
                    italic: Some(false),
                },
                Self {
                    variant,
                    bold: false,
                    italic: Some(true),
                },
                Self {
                    variant,
                    bold: true,
                    italic: None,
                },
                Self {
                    variant,
                    bold: true,
                    italic: Some(false),
                },
                Self {
                    variant,
                    bold: true,
                    italic: Some(true),
                },
            ]
        })
    }

    fn to_typst(self, body: impl Display) -> String {
        let mut s = match self.variant {
            MathVariant::Plain => body.to_string(),
            MathVariant::Fraktur => format!("frak({body})"),
            MathVariant::SansSerif => format!("sans({body})"),
            MathVariant::Monospace => format!("mono({body})"),
            MathVariant::DoubleStruck => format!("bb({body})"),
            MathVariant::Chancery => format!("cal({body})"),
            MathVariant::Roundhand => format!("scr({body})"),
            _ => unreachable!(),
        };
        s = match self.italic {
            None => s,
            Some(false) => format!("upright({s})"),
            Some(true) => format!("italic({s})"),
        };
        if self.bold {
            s = format!("bold({s})")
        }
        s
    }

    fn apply_to(self, c: char) -> String {
        let style = MathStyle::select(c, Some(self.variant), self.bold, self.italic);
        to_style(c, style).collect()
    }

    fn find_variations(source: &str) -> impl Iterator<Item = (Self, String)> {
        let mut found = HashSet::new();
        Self::iter().flat_map(move |config| {
            let res = source
                .chars()
                .map(|c| config.apply_to(c))
                .collect::<String>();
            let mut new = Vec::new();
            // If the result contains a text presentation selector, we consider
            // that we can obtain the version without the selector because math
            // fonts have a text presentation by default.
            if let Some(res_alt) = res.strip_suffix("\u{FE0E}")
                && !found.contains(res_alt)
            {
                found.insert(res_alt.to_owned());
                new.push((config, res_alt.to_owned()))
            }
            // Add the raw result as well.
            if !found.contains(&res) {
                found.insert(res.clone());
                new.push((config, res))
            }
            new
        })
    }
}

fn find_math_names() -> Vec<(String, String)> {
    let mut symbols = Vec::new();
    explore_module(&mut symbols, &mut Vec::new(), codex::SYM, "", false);

    // We want to list everything that can be obtained by styling a Latin
    // letter, a digit, a Typst shorthand, or a Codex symbol.
    let bases = ('0'..='9')
        .chain('A'..='Z')
        .chain('a'..='z')
        .map(|c| (c.to_string(), c.to_string()))
        .chain(
            MathShorthand::LIST
                .iter()
                .map(|&(shorthand, value)| (shorthand.to_owned(), value.to_string())),
        )
        .chain(symbols);

    let mut math_names = Vec::new();
    for (name, value) in bases {
        math_names.extend(
            MathStyleConfig::find_variations(&value)
                .map(|(config, value)| (config.to_typst(&name), value)),
        )
    }
    math_names
}

fn encode_name_list(buf: &mut String, name: &str, symbols: Vec<(String, String)>) {
    let mut symbols_by_value = HashMap::<_, Vec<_>>::new();
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
            (value.into_bytes(), encoded_names)
        })
        .collect::<Vec<_>>();
    encoded_symbol_list.sort_by_key(|(value, _)| value.clone());

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
    let math_names = find_math_names();

    let mut buf = String::new();
    encode_name_list(&mut buf, "SYMBOLS", symbols);
    encode_name_list(&mut buf, "DEPRECATED", deprecated);
    encode_name_list(&mut buf, "MATH_NAMES", math_names);

    let out = std::env::var_os("OUT_DIR").unwrap();
    let dest = Path::new(&out).join("out.rs");
    std::fs::write(&dest, buf).unwrap();
}
