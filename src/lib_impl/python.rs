//! Cargo.toml of the maturin project of the Python target.

use toml_edit::{value, Array, DocumentMut, InlineTable, Item, Value};

use super::compiler::BITIS_COMPILER_VERSION;

/// pyo3 version the generated Python bindings are written for (`Python::attach`,
/// `#[pyclass(from_py_object)]`: pyo3 >= 0.27). The compiler sets it in the Cargo.toml of the
/// maturin project, because the generated code does not compile with other pyo3 versions.
pub const PYO3_VERSION: &str = "0.29";
/// Stable ABI feature set if the project does not select an `abi3*` feature itself:
/// one wheel for CPython >= 3.8 (matches `requires-python = ">=3.8"` of `maturin init`).
pub const PYO3_ABI3_FEATURE: &str = "abi3-py38";
/// Version of strum_macros used by the generated Rust code (`messages.rs`).
pub const STRUM_MACROS_VERSION: &str = "0.27";

/// Updates the Cargo.toml of the maturin project so that the generated code builds:
/// - `package.version` = bitis compiler version (= version of the wheel),
/// - `pyo3` = [`PYO3_VERSION`] with an `abi3*` feature (default [`PYO3_ABI3_FEATURE`]; other features are kept),
/// - `strum_macros` is added if missing,
/// - `bitis` = compiler version if `set_bitis_dependency` (otherwise the bitis dependency is left alone).
pub fn update_python_cargo_toml(content: &str, set_bitis_dependency: bool) -> Result<String, String> {
    let mut doc = content.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    let package = doc.get_mut("package").and_then(|p| p.as_table_like_mut()).ok_or("no [package] table")?;
    package.insert("version", value(BITIS_COMPILER_VERSION));

    if doc.get("dependencies").is_none() { doc["dependencies"] = toml_edit::table(); }
    let deps = doc["dependencies"].as_table_like_mut().ok_or("[dependencies] is not a table")?;

    // pyo3: keep the features, replace the version, make sure an abi3 feature is selected
    let mut features: Vec<String> = deps.get("pyo3").and_then(|p| p.as_table_like())
        .and_then(|t| t.get("features")).and_then(|f| f.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    if !features.iter().any(|f| f.starts_with("abi3")) { features.push(PYO3_ABI3_FEATURE.to_string()); }
    match deps.get_mut("pyo3").and_then(|p| p.as_table_like_mut()) {
        Some(t) => {
            t.insert("version", value(PYO3_VERSION));
            t.insert("features", value(features.iter().collect::<Array>()));
        }
        None => {
            let mut t = InlineTable::new();
            t.insert("version", Value::from(PYO3_VERSION));
            t.insert("features", Value::Array(features.iter().collect::<Array>()));
            deps.insert("pyo3", Item::Value(Value::InlineTable(t)));
        }
    }
    if deps.get("strum_macros").is_none() { deps.insert("strum_macros", value(STRUM_MACROS_VERSION)); }
    if set_bitis_dependency { deps.insert("bitis", value(BITIS_COMPILER_VERSION)); }
    Ok(doc.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_maturin_init() {
        let t = "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[lib]\nname = \"x\"\n\n[dependencies]\npyo3 = \"0.30.0\"\n";
        let r = update_python_cargo_toml(t, true).unwrap();
        assert!(r.contains(&format!("version = \"{BITIS_COMPILER_VERSION}\"")), "{r}");
        assert!(r.contains("pyo3 = { version = \"0.29\", features = [\"abi3-py38\"] }"), "{r}");
        assert!(r.contains("strum_macros = \"0.27\""), "{r}");
        assert!(r.contains(&format!("bitis = \"{BITIS_COMPILER_VERSION}\"")), "{r}");
        // idempotent
        assert_eq!(update_python_cargo_toml(&r, true).unwrap(), r);
    }

    #[test]
    fn keeps_features_and_bitis_path() {
        let t = "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\n\
                 pyo3 = { version = \"0.25.0\", features = [\"extension-module\", \"abi3-py310\"] }\n\
                 bitis = { path = \"../bitis\" }\nstrum_macros = \"0.27.2\"\n";
        let r = update_python_cargo_toml(t, false).unwrap();
        assert!(r.contains("pyo3 = { version = \"0.29\", features = [\"extension-module\", \"abi3-py310\"] }"), "{r}");
        assert!(r.contains("bitis = { path = \"../bitis\" }"), "{r}");
        assert!(r.contains("strum_macros = \"0.27.2\""), "{r}");
    }

    #[test]
    fn errors() {
        assert!(update_python_cargo_toml("[dependencies]\n", true).is_err());
        assert!(update_python_cargo_toml("not toml [", true).is_err());
    }
}
