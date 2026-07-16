//! CLI-facing configuration: a thin wrapper that loads a [`FeatureSpec`] from a
//! JSON or TOML file body.

use crate::error::Result;
use crate::spec::FeatureSpec;
use serde::{Deserialize, Serialize};

/// A loaded configuration: just the feature spec, for now.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    /// The feature spec to build.
    pub spec: FeatureSpec,
}

impl Config {
    /// Load a config whose body is a bare [`FeatureSpec`] in JSON.
    ///
    /// # Errors
    /// Returns [`crate::Error::Parse`] on malformed JSON.
    pub fn from_json(s: &str) -> Result<Self> {
        Ok(Self {
            spec: FeatureSpec::from_json(s)?,
        })
    }

    /// Load a config whose body is a bare [`FeatureSpec`] in TOML.
    ///
    /// # Errors
    /// Returns [`crate::Error::Parse`] on malformed TOML.
    pub fn from_toml(s: &str) -> Result<Self> {
        Ok(Self {
            spec: FeatureSpec::from_toml(s)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_spec_from_json() {
        let json = r#"{ "universe": ["AAA"],
            "features": [ {"kind":"price","field":"close"} ] }"#;
        let cfg = Config::from_json(json).unwrap();
        assert_eq!(cfg.spec.columns(), vec!["price.close"]);
    }

    #[test]
    fn loads_spec_from_toml() {
        let toml = r#"
            universe = ["AAA"]
            [[features]]
            kind = "price"
            field = "close"
        "#;
        let cfg = Config::from_toml(toml).unwrap();
        assert_eq!(cfg.spec.columns(), vec!["price.close"]);
    }
}
