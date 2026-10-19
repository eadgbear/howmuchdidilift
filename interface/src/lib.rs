use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub pid: String,
    pub name: String,
    pub is_verified: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrentResponse {
    pub pid: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterParams {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyParams {
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForgotParams {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResetParams {
    pub token: String,
    pub password: String,
}

/// One searchable emoji.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmojiEntry {
    /// Codepoint token used in a measure's icon (e.g. `1f34c`, `1f1e9_1f1ea`).
    pub codepoint: String,
    /// Native glyph, for display in the picker.
    pub glyph: String,
    /// Human-readable name, searched against.
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MeasureCreate {
    pub name: String,
    pub grams: f64,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Measure {
    pub id: i32,
    pub name: String,
    pub grams: f64,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumIter, EnumString, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum InputWeightType {
    Lbs,
    Kgs,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RandomWeightRequest {
    pub input_amt: f64,
    pub input_type: InputWeightType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RandomWeightResponse {
    pub when: DateTime<Utc>,
    pub input_amt: String,
    pub input_type: InputWeightType,
    pub output_weight: String,
    pub units: String,
    #[serde(default)]
    pub icon: Option<String>,
    /// Canonical share URL, e.g. `howmuchdidilift.com/w/225lbs`.
    #[serde(default)]
    pub permalink: String,
    /// Ready-to-copy social sentence including the permalink.
    #[serde(default)]
    pub share_text: String,
}
