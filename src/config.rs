#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    #[serde(default = "default_true")]
    pub dump_method: bool,
    #[serde(default = "default_true")]
    pub dump_field: bool,
    #[serde(default = "default_false")]
    pub dump_property: bool,
    #[serde(default = "default_false")]
    pub dump_attribute: bool,
    #[serde(default = "default_true")]
    pub dump_field_offset: bool,
    #[serde(default = "default_true")]
    pub dump_method_offset: bool,
    #[serde(default = "default_true")]
    pub dump_type_def_index: bool,
    #[serde(default = "default_true")]
    pub generate_dummy_dll: bool,
    #[serde(default = "default_true")]
    pub generate_struct: bool,
    #[serde(default = "default_true")]
    pub dummy_dll_add_token: bool,
    #[serde(default = "default_true")]
    pub require_any_key: bool,
    #[serde(default = "default_false")]
    pub force_il2cpp_version: bool,
    #[serde(default = "default_force_version")]
    pub force_version: f64,
    #[serde(default = "default_false")]
    pub force_dump: bool,
    #[serde(default = "default_false")]
    pub no_redirected_pointer: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dump_method: true,
            dump_field: true,
            dump_property: false,
            dump_attribute: false,
            dump_field_offset: true,
            dump_method_offset: true,
            dump_type_def_index: true,
            generate_dummy_dll: true,
            generate_struct: true,
            dummy_dll_add_token: true,
            require_any_key: true,
            force_il2cpp_version: false,
            force_version: 24.3,
            force_dump: false,
            no_redirected_pointer: false,
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_force_version() -> f64 {
    24.3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.dump_method);
        assert!(config.dump_field);
        assert!(!config.dump_property);
    }

    #[test]
    fn test_deserialize_config() {
        let json = r#"{
            "DumpMethod": false,
            "DumpField": true,
            "ForceVersion": 27.2
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.dump_method);
        assert!(config.dump_field);
        assert_eq!(config.force_version, 27.2);
    }
}
