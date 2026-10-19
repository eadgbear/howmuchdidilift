//! Abstraction over sources of measures ("fact stores").
//!
//! Each store returns the measures whose names match a set of search terms (or
//! all of them when there are no terms). [`pick_measure`] pools every store's
//! candidates and picks one according to the requested [`MatchType`]. New
//! sources (a remote API, a themed pack, …) just implement [`FactStore`] and get
//! added to [`stores`].

use interface::{MatchType, Measure};
use loco_rs::prelude::*;
use rand::seq::SliceRandom;
use sea_orm::{
    sea_query::{extension::postgres::PgExpr, Expr},
    Condition, DatabaseConnection, QueryFilter,
};

use crate::models::_entities::measures::{Column, Entity};

/// AtLeast: only measures where `lift / measure` is at most this are "close".
const AT_LEAST_MARGIN: f64 = 1.5;
/// AtLeast: pick randomly among at most this many of the closest measures.
const AT_LEAST_TOP_N: usize = 5;

#[async_trait]
pub trait FactStore: Send + Sync {
    /// Measures whose name contains any of `terms` (case-insensitive), or every
    /// measure when `terms` is empty.
    async fn matching(&self, db: &DatabaseConnection, terms: &[String]) -> Result<Vec<Measure>>;
}

/// Built-in measures from `config/measures.toml`.
struct ConfigStore;

#[async_trait]
impl FactStore for ConfigStore {
    async fn matching(&self, _db: &DatabaseConnection, terms: &[String]) -> Result<Vec<Measure>> {
        Ok(filter_by_name(crate::measures_config::defaults(), terms))
    }
}

/// In-memory equivalent of `name ILIKE '%term%' OR …`.
fn filter_by_name(measures: &[Measure], terms: &[String]) -> Vec<Measure> {
    let terms: Vec<String> = terms.iter().map(|t| t.to_lowercase()).collect();
    measures
        .iter()
        .filter(|m| {
            let name = m.name.to_lowercase();
            terms.is_empty() || terms.iter().any(|t| name.contains(t.as_str()))
        })
        .cloned()
        .collect()
}

/// User-added measures stored in the database.
struct DbStore;

#[async_trait]
impl FactStore for DbStore {
    async fn matching(&self, db: &DatabaseConnection, terms: &[String]) -> Result<Vec<Measure>> {
        let mut query = Entity::find();
        if !terms.is_empty() {
            let cond = terms.iter().fold(Condition::any(), |cond, t| {
                cond.add(Expr::col(Column::Name).ilike(format!("%{}%", escape_like(t))))
            });
            query = query.filter(cond);
        }
        Ok(query.all(db).await?.into_iter().map(Into::into).collect())
    }
}

/// Escape LIKE wildcards so search terms match literally.
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// All registered fact stores.
fn stores() -> Vec<Box<dyn FactStore>> {
    vec![Box::new(ConfigStore), Box::new(DbStore)]
}

async fn gather(db: &DatabaseConnection, terms: &[String]) -> Result<Vec<Measure>> {
    let mut all = Vec::new();
    for store in stores() {
        all.extend(store.matching(db, terms).await?);
    }
    Ok(all)
}

/// The closest measures not heavier than the lift, within [`AT_LEAST_MARGIN`]
/// and capped at [`AT_LEAST_TOP_N`]. Falls back to the single nearest lighter
/// measure, then to the lightest measure overall if everything is heavier.
fn at_least_candidates(lift_grams: f64, measures: &[Measure]) -> Vec<&Measure> {
    let mut below: Vec<&Measure> = measures
        .iter()
        .filter(|m| m.grams > 0.0 && m.grams <= lift_grams)
        .collect();
    below.sort_by(|a, b| b.grams.total_cmp(&a.grams));

    let close: Vec<&Measure> = below
        .iter()
        .copied()
        .filter(|m| lift_grams / m.grams <= AT_LEAST_MARGIN)
        .take(AT_LEAST_TOP_N)
        .collect();
    if !close.is_empty() {
        return close;
    }
    if let Some(nearest) = below.first() {
        return vec![nearest];
    }
    measures
        .iter()
        .filter(|m| m.grams > 0.0)
        .min_by(|a, b| a.grams.total_cmp(&b.grams))
        .into_iter()
        .collect()
}

/// Pick a measure for a lift of `lift_grams`.
///
/// Candidates are every measure whose name matches any of `terms` (all measures
/// if `terms` is empty or nothing matches). `Mult` picks uniformly at random;
/// `AtLeast` picks randomly among the closest measures not heavier than the lift.
pub async fn pick_measure(
    db: &DatabaseConnection,
    lift_grams: f64,
    match_type: &MatchType,
    terms: &[String],
) -> Result<Measure> {
    let terms: Vec<String> = terms
        .iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    let mut candidates = gather(db, &terms).await?;
    if candidates.is_empty() && !terms.is_empty() {
        candidates = gather(db, &[]).await?;
    }

    let mut rng = rand::thread_rng();
    let picked = match match_type {
        MatchType::Mult => candidates.choose(&mut rng),
        MatchType::AtLeast => at_least_candidates(lift_grams, &candidates)
            .choose(&mut rng)
            .copied(),
    };
    picked.cloned().ok_or_else(|| Error::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(name: &str, grams: f64) -> Measure {
        Measure {
            name: name.to_string(),
            grams,
            ..Default::default()
        }
    }

    fn names(ms: &[&Measure]) -> Vec<String> {
        ms.iter().map(|m| m.name.clone()).collect()
    }

    #[test]
    fn at_least_keeps_close_lighter_measures() {
        let ms = [m("a", 50.0), m("b", 80.0), m("c", 95.0), m("d", 120.0)];
        assert_eq!(names(&at_least_candidates(100.0, &ms)), ["c", "b"]);
    }

    #[test]
    fn at_least_caps_at_top_n() {
        let ms: Vec<Measure> = (0..10)
            .map(|i| m(&i.to_string(), 90.0 + f64::from(i)))
            .collect();
        let picked = at_least_candidates(100.0, &ms);
        assert_eq!(picked.len(), AT_LEAST_TOP_N);
        assert_eq!(picked[0].name, "9");
    }

    #[test]
    fn at_least_falls_back_to_nearest_below() {
        let ms = [m("tiny", 1.0), m("small", 10.0), m("huge", 1000.0)];
        assert_eq!(names(&at_least_candidates(100.0, &ms)), ["small"]);
    }

    #[test]
    fn at_least_falls_back_to_lightest_when_all_heavier() {
        let ms = [m("big", 500.0), m("bigger", 200.0)];
        assert_eq!(names(&at_least_candidates(100.0, &ms)), ["bigger"]);
    }

    #[test]
    fn name_filter_is_case_insensitive_or() {
        let ms = [m("House Cats", 1.0), m("Dogs", 1.0), m("Bananas", 1.0)];
        let terms = ["CAT".to_string(), "dog".to_string()];
        let got: Vec<String> = filter_by_name(&ms, &terms)
            .into_iter()
            .map(|m| m.name)
            .collect();
        assert_eq!(got, ["House Cats", "Dogs"]);
        assert_eq!(filter_by_name(&ms, &[]).len(), 3);
    }

    #[test]
    fn like_wildcards_are_escaped() {
        assert_eq!(escape_like(r"50%_\"), r"50\%\_\\");
    }
}
