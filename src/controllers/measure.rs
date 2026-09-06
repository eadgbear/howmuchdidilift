#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use axum::{
    extract::Query,
    http::{header, HeaderName, HeaderValue},
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use serde::Deserialize;

use interface::{
    InputWeightType, Measure, MeasureCreate, RandomWeightRequest, RandomWeightResponse,
};
use loco_rs::prelude::*;
use sea_orm::{prelude::DateTimeUtc, TryIntoModel};

use crate::models::{
    _entities::measures::{ActiveModel, Entity, Model},
    users,
};
use numfmt::{Formatter, Precision};

impl From<Model> for Measure {
    fn from(model: Model) -> Self {
        Measure {
            id: model.id,
            name: model.name,
            grams: model.grams,
            icon: model.icon,
        }
    }
}

async fn load_item(auth: auth::JWT, ctx: &AppContext, id: i32) -> Result<Model> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

pub async fn list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Json<Vec<Measure>>> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(
        Entity::find()
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    )
}

pub async fn add(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<MeasureCreate>,
) -> Result<Json<Measure>> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(
        ActiveModel::create(&ctx.db, params)
            .await?
            .try_into_model()?
            .into(),
    )
}

pub async fn update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Measure>,
) -> Result<Json<Measure>> {
    let item = load_item(auth, &ctx, id).await?;
    let mut item = item.into_active_model();
    item.name = Set(params.name);
    item.updated_at = Set(DateTimeUtc::default().naive_local());
    item.grams = Set(params.grams);
    item.icon = Set(params.icon);
    let item = item.update(&ctx.db).await?;
    format::json(item.into())
}

pub async fn remove(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<()> {
    load_item(auth, &ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub async fn get_one(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Json<Measure>> {
    format::json(load_item(auth, &ctx, id).await?.into())
}

fn grams_multiplier(t: &InputWeightType) -> f64 {
    match t {
        InputWeightType::Lbs => 453.592,
        InputWeightType::Kgs => 1000.0,
    }
}

/// Format a comparison count, never in scientific notation. Large counts get
/// thousands separators; tiny fractions show all their leading zeroes plus a few
/// significant digits (that's the joke for country-sized measures).
fn fmt_count(v: f64) -> String {
    if !v.is_finite() || v <= 0.0 {
        return "0".to_string();
    }
    if v >= 1.0 {
        let mut f = Formatter::new()
            .precision(Precision::Decimals(2))
            .separator(',')
            .unwrap();
        return f.fmt2(v).to_string();
    }
    // v < 1: enough decimals for the leading zeroes + ~3 significant figures.
    let places = usize::try_from(((-v.log10()).ceil() as i32 + 3).clamp(2, 24)).unwrap_or(2);
    format!("{v:.places$}")
}

/// Everything needed to render a card or answer a convert, computed once.
struct LiftResult {
    count: String,
    units: String,
    icon: Option<String>,
    primary: String,
    secondary: String,
    permalink: String,
    share_text: String,
}

/// Compute a lift result for `amt`/`unit` against a specific `measure`.
fn build_lift_result(amt: f64, unit: &InputWeightType, measure: &Measure) -> LiftResult {
    let source_grams = amt * grams_multiplier(unit);
    let div = source_grams / measure.grams;
    // Show both units; use the entered amount verbatim for its own unit (no
    // lossy round-trip through grams), convert only for the other.
    let (pounds, kgs) = match unit {
        InputWeightType::Lbs => (amt, source_grams / grams_multiplier(&InputWeightType::Kgs)),
        InputWeightType::Kgs => (source_grams / grams_multiplier(&InputWeightType::Lbs), amt),
    };

    let mut f = Formatter::new()
        .precision(Precision::Decimals(2))
        .separator(',')
        .unwrap();
    let count = fmt_count(div);
    let lbs_line = format!("{} lbs", f.fmt2(pounds));
    let kgs_line = format!("{} kgs", f.fmt2(kgs));
    // Lead with whichever unit the user entered.
    let (primary, secondary) = match unit {
        InputWeightType::Lbs => (lbs_line, kgs_line),
        InputWeightType::Kgs => (kgs_line, lbs_line),
    };

    let permalink = format!(
        "howmuchdidilift.com/w/{}",
        crate::controllers::pages::spec_token(amt, unit)
    );
    let units = measure.name.clone();
    let share_text =
        format!("Today's workout was {primary} ({secondary}), or {count} {units}!\n\n{permalink}");

    LiftResult {
        count,
        units,
        icon: measure.icon.clone(),
        primary,
        secondary,
        permalink,
        share_text,
    }
}

#[debug_handler]
pub async fn convert(
    State(ctx): State<AppContext>,
    Json(params): Json<RandomWeightRequest>,
) -> Result<Json<RandomWeightResponse>> {
    let measure = crate::fact_store::random_measure(&ctx.db).await?;
    let r = build_lift_result(params.input_amt, &params.input_type, &measure);

    let mut f = Formatter::new()
        .precision(Precision::Decimals(2))
        .separator(',')
        .unwrap();

    Ok(Json(RandomWeightResponse {
        when: Utc::now(),
        input_amt: f.fmt2(params.input_amt).to_string(),
        input_type: params.input_type,
        output_weight: r.count,
        units: r.units,
        icon: r.icon,
        permalink: r.permalink,
        share_text: r.share_text,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CardParams {
    pub amt: f64,
    #[serde(default = "default_unit")]
    pub unit: InputWeightType,
}

fn default_unit() -> InputWeightType {
    InputWeightType::Lbs
}

/// Renders a shareable PNG card for a given lift (always a fresh random measure).
///
/// The matching share sentence rides along in the `x-share-text` header so the
/// client can copy text matching the image from one fetch. Public (no auth) so
/// social scrapers can fetch it as an `og:image`.
pub async fn card(
    State(ctx): State<AppContext>,
    Query(params): Query<CardParams>,
) -> Result<Response> {
    let measure = crate::fact_store::random_measure(&ctx.db).await?;
    let r = build_lift_result(params.amt, &params.unit, &measure);

    let png = crate::card::render_card(
        &r.primary,
        &r.secondary,
        &r.count,
        &r.units,
        &r.permalink,
        r.icon.as_deref(),
    )
    .map_err(|e| Error::string(&e))?;

    // Random each call, so don't cache.
    let mut resp = (
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        png,
    )
        .into_response();
    // base64 so the (multi-line, possibly non-ASCII) sentence fits in a header.
    if let Ok(hv) = HeaderValue::from_str(&STANDARD.encode(&r.share_text)) {
        resp.headers_mut()
            .insert(HeaderName::from_static("x-share-text"), hv);
    }
    Ok(resp)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("measures")
        .add("/", get(list))
        .add("/", post(add))
        .add("/convert", post(convert))
        .add("/card", get(card))
        .add("/:id", get(get_one))
        .add("/:id", delete(remove))
        .add("/:id", post(update))
}
