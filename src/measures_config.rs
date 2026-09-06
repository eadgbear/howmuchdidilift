//! Default measures loaded from `config/measures.toml` at runtime.
//!
//! These are the built-in comparison units. The database only holds measures a
//! user adds dynamically via the editor; conversions draw a random measure from
//! both pools combined.

use std::sync::OnceLock;

use interface::Measure;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct File {
    #[serde(default)]
    measures: Vec<Def>,
}

#[derive(Debug, Deserialize)]
struct Def {
    name: String,
    grams: f64,
    /// One or two Noto codepoint tokens; joined with `+` for storage/rendering.
    #[serde(default)]
    icon: Vec<String>,
}

static MEASURES: OnceLock<Vec<Measure>> = OnceLock::new();

/// The built-in measures, parsed once from `config/measures.toml`.
pub fn defaults() -> &'static [Measure] {
    MEASURES.get_or_init(|| {
        let text = match std::fs::read_to_string("config/measures.toml") {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("could not read config/measures.toml: {e}");
                return Vec::new();
            }
        };
        let file: File = match toml::from_str(&text) {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("could not parse config/measures.toml: {e}");
                return Vec::new();
            }
        };
        file.measures
            .into_iter()
            .enumerate()
            .map(|(i, d)| Measure {
                id: -(i32::try_from(i).unwrap_or(i32::MAX) + 1),
                name: d.name,
                grams: d.grams,
                icon: (!d.icon.is_empty()).then(|| d.icon.join("+")),
            })
            .collect()
    })
}

/// Look up a config measure by its (negative) id.
pub fn by_id(id: i32) -> Option<Measure> {
    if id >= 0 {
        return None;
    }
    let idx = usize::try_from(-id - 1).ok()?;
    defaults().get(idx).cloned()
}
