use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// A component manifest compatible with the Dioxus Components installer format.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Component {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) component_dependencies: Vec<ComponentDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) cargo_dependencies: Vec<CargoDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) members: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) exclude: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) global_assets: Vec<String>,
}

/// A dependency on another component, either builtin or third-party.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub(crate) enum ComponentDependency {
    Builtin(String),
    ThirdParty {
        name: String,
        git: String,
        #[serde(default)]
        rev: Option<String>,
    },
}

/// A dependency on a cargo crate required for a component.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub(crate) enum CargoDependency {
    Simple(String),
    Detailed {
        name: String,
        #[serde(default)]
        version: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        features: Vec<String>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        default_features: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        git: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rev: Option<String>,
    },
}

impl CargoDependency {
    /// Returns the dependency name for diagnostics.
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Simple(name) => name,
            Self::Detailed { name, .. } => name,
        }
    }

    /// Builds `cargo add` arguments for this dependency.
    pub(crate) fn add_command_args(&self) -> Vec<String> {
        match self {
            Self::Simple(name) => vec!["add".to_string(), name.clone()],
            Self::Detailed {
                name,
                version,
                features,
                default_features,
                git,
                rev,
            } => {
                let mut args = Vec::new();
                args.push("add".to_string());

                let dep_spec = if let Some(version) = version {
                    format!("{name}@{version}")
                } else {
                    name.clone()
                };
                args.push(dep_spec);

                if !features.is_empty() {
                    args.push("--features".to_string());
                    args.push(features.join(","));
                }

                if !*default_features {
                    args.push("--no-default-features".to_string());
                }

                if let Some(git) = git {
                    args.push("--git".to_string());
                    args.push(git.clone());
                }

                if let Some(rev) = rev {
                    args.push("--rev".to_string());
                    args.push(rev.clone());
                }

                args
            }
        }
    }
}

/// Returns a JSON Schema payload for component manifests.
pub(crate) fn component_manifest_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Component",
        "type": "object",
        "additionalProperties": false,
        "required": ["name"],
        "properties": {
            "name": { "type": "string" },
            "description": { "type": "string", "default": "" },
            "authors": {
                "type": "array",
                "items": { "type": "string" },
                "default": []
            },
            "componentDependencies": {
                "type": "array",
                "items": {
                    "anyOf": [
                        { "type": "string" },
                        {
                            "type": "object",
                            "required": ["name", "git"],
                            "additionalProperties": false,
                            "properties": {
                                "name": { "type": "string" },
                                "git": { "type": "string" },
                                "rev": { "type": "string" }
                            }
                        }
                    ]
                },
                "default": []
            },
            "cargoDependencies": {
                "type": "array",
                "items": {
                    "anyOf": [
                        { "type": "string" },
                        {
                            "type": "object",
                            "required": ["name"],
                            "additionalProperties": false,
                            "properties": {
                                "name": { "type": "string" },
                                "version": { "type": "string" },
                                "features": {
                                    "type": "array",
                                    "items": { "type": "string" },
                                    "default": []
                                },
                                "defaultFeatures": { "type": "boolean", "default": false },
                                "git": { "type": "string" },
                                "rev": { "type": "string" }
                            }
                        }
                    ]
                },
                "default": []
            },
            "members": {
                "type": "array",
                "items": { "type": "string" },
                "default": []
            },
            "exclude": {
                "type": "array",
                "items": { "type": "string" },
                "default": []
            },
            "globalAssets": {
                "type": "array",
                "items": { "type": "string" },
                "default": []
            }
        }
    })
}

/// Renders schema output with stable top-level key ordering.
pub(crate) fn render_component_manifest_schema_pretty() -> anyhow::Result<String> {
    let value = component_manifest_schema();
    let mut ordered = BTreeMap::new();
    if let Some(object) = value.as_object() {
        for (key, field) in object {
            ordered.insert(key.clone(), field.clone());
        }
    }
    serde_json::to_string_pretty(&ordered).map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_simple_cargo_add_args() {
        let args = CargoDependency::Simple("serde".to_string()).add_command_args();
        assert_eq!(args, vec!["add", "serde"]);
    }

    #[test]
    fn builds_detailed_cargo_add_args() {
        let args = CargoDependency::Detailed {
            name: "dioxus-primitives".to_string(),
            version: None,
            features: vec!["router".to_string()],
            default_features: false,
            git: Some("https://github.com/DioxusLabs/components".to_string()),
            rev: Some("abc123".to_string()),
        }
        .add_command_args();

        assert_eq!(
            args,
            vec![
                "add",
                "dioxus-primitives",
                "--features",
                "router",
                "--no-default-features",
                "--git",
                "https://github.com/DioxusLabs/components",
                "--rev",
                "abc123"
            ]
        );
    }

    #[test]
    fn schema_includes_required_name() {
        let schema = component_manifest_schema();
        let required = schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required array");
        assert!(required.iter().any(|item| item == "name"));
    }
}
