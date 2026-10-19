//! Emoji search for the measures editor's icon picker.
//!
//! The full index (~1.7k entries) is loaded once server-side; the endpoint
//! returns only the matches for a query, so the client never downloads it all.

#![allow(clippy::missing_errors_doc)]

use std::sync::OnceLock;

use axum::extract::Query;
use interface::EmojiEntry;
use loco_rs::prelude::*;
use serde::Deserialize;

static INDEX: OnceLock<Vec<EmojiEntry>> = OnceLock::new();

fn index() -> &'static [EmojiEntry] {
    INDEX.get_or_init(
        || match std::fs::read_to_string("assets/emoji-index.json") {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
                tracing::error!("bad assets/emoji-index.json: {e}");
                Vec::new()
            }),
            Err(e) => {
                tracing::warn!("could not read assets/emoji-index.json: {e}");
                Vec::new()
            }
        },
    )
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    #[serde(default)]
    pub q: String,
    pub limit: Option<usize>,
}

/// `GET /api/emoji?q=banana&limit=48` — name substring search.
pub async fn search(Query(params): Query<SearchParams>) -> Result<Json<Vec<EmojiEntry>>> {
    let term = params.q.trim().to_lowercase();
    let limit = params.limit.unwrap_or(48).min(200);
    let matches = index()
        .iter()
        .filter(|e| term.is_empty() || e.name.to_lowercase().contains(&term))
        .take(limit)
        .cloned()
        .collect();
    format::json(matches)
}

pub fn routes() -> Routes {
    Routes::new().prefix("emoji").add("/", get(search))
}
