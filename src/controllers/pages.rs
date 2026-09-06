//! Human/scraper-facing HTML pages served outside the `/api` prefix.
//!
//! The `/w/:spec` permalink (e.g. `/w/225lbs`, `/w/100kg`) renders a small HTML
//! page carrying OpenGraph/Twitter meta so the result unfurls into a share card
//! on Discord/iMessage/Twitter. The `og:image` points at the PNG endpoint.

#![allow(clippy::missing_errors_doc)]

use axum::{
    extract::{Host, Path},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use interface::InputWeightType;
use loco_rs::prelude::*;

/// Social/preview crawlers that should receive the OG HTML. Everyone else is a
/// real browser and gets redirected into the interactive app.
const CRAWLER_UAS: [&str; 11] = [
    "discordbot",
    "facebookexternalhit",
    "twitterbot",
    "slackbot",
    "whatsapp",
    "telegrambot",
    "linkedinbot",
    "redditbot",
    "googlebot",
    "bingbot",
    "embedly",
];

fn is_crawler(headers: &HeaderMap) -> bool {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(str::to_lowercase)
        .is_some_and(|ua| CRAWLER_UAS.iter().any(|bot| ua.contains(bot)))
}

/// Compact URL token for a weight, e.g. `225lbs` / `100.5kg`. Whole numbers
/// drop the decimal point for cleaner links.
pub fn spec_token(amt: f64, unit: &InputWeightType) -> String {
    let n = if amt.fract() == 0.0 {
        format!("{amt:.0}")
    } else {
        format!("{amt}")
    };
    format!("{n}{}", unit.to_string().to_lowercase())
}

/// Parse a path spec like `225lbs`, `225lb`, `100kg`, `100kgs` into an amount
/// and unit. A bare number defaults to pounds. Returns `None` on garbage.
pub fn parse_weight_spec(spec: &str) -> Option<(f64, InputWeightType)> {
    let spec = spec.trim().to_lowercase();
    let split = spec
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(spec.len());
    let (num, unit) = spec.split_at(split);

    let amt: f64 = num.parse().ok()?;
    if !amt.is_finite() || amt <= 0.0 {
        return None;
    }

    let unit = match unit.trim() {
        "" | "lb" | "lbs" | "pound" | "pounds" => InputWeightType::Lbs,
        "kg" | "kgs" | "kilo" | "kilos" | "kilogram" | "kilograms" => InputWeightType::Kgs,
        _ => return None,
    };
    Some((amt, unit))
}

/// Best-effort absolute base URL from proxy headers, falling back to the Host.
fn base_url(host: &str, headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    format!("{scheme}://{host}")
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `GET /w/:spec` — share permalink. Crawlers (Discord, Twitter, …) get an
/// OG-tagged HTML page so the card unfurls; real browsers are redirected into
/// the interactive app pre-filled with the weight.
pub async fn weight_page(
    Host(host): Host,
    headers: HeaderMap,
    Path(spec): Path<String>,
) -> Result<Response> {
    let Some((amt, unit)) = parse_weight_spec(&spec) else {
        return Ok((
            StatusCode::BAD_REQUEST,
            Html("<h1>Invalid weight</h1><p>Try /w/225lbs or /w/100kg</p>".to_string()),
        )
            .into_response());
    };

    let token = spec_token(amt, &unit);

    // Humans → the interactive app, pre-filled via the `w` query param.
    if !is_crawler(&headers) {
        return Ok(Redirect::to(&format!("/?w={token}")).into_response());
    }

    // Crawlers → OG/Twitter/Discord card unfurl.
    let base = base_url(&host, &headers);
    let unit_l = unit.to_string().to_lowercase();
    let card_url = format!("{base}/api/measures/card?amt={amt}&unit={unit_l}");
    let page_url = format!("{base}/w/{}", esc(&token));
    let (cw, ch) = crate::card::dimensions();

    // Both units in the description; the fun payload is the card image itself.
    let grams = amt
        * match unit {
            InputWeightType::Lbs => 453.592,
            InputWeightType::Kgs => 1000.0,
        };
    let title = format!("I lifted {amt} {unit_l} — how much is that really?");
    let description = format!(
        "{:.0} lbs / {:.1} kgs — but how much is that in bananas, cats, and elephants?",
        grams / 453.592,
        grams / 1000.0,
    );

    let html = format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <meta name="theme-color" content="#f472b6"/>
  <title>{title}</title>
  <meta name="description" content="{description}"/>
  <meta property="og:type" content="website"/>
  <meta property="og:site_name" content="howmuchdidilift"/>
  <meta property="og:title" content="{title}"/>
  <meta property="og:description" content="{description}"/>
  <meta property="og:image" content="{card_url}"/>
  <meta property="og:image:width" content="{cw}"/>
  <meta property="og:image:height" content="{ch}"/>
  <meta property="og:url" content="{page_url}"/>
  <meta name="twitter:card" content="summary_large_image"/>
  <meta name="twitter:title" content="{title}"/>
  <meta name="twitter:description" content="{description}"/>
  <meta name="twitter:image" content="{card_url}"/>
  <style>
    body {{ margin:0; background:#1b1f2a; color:#c3c9d6; font-family:system-ui,sans-serif;
           min-height:100vh; display:flex; flex-direction:column; align-items:center;
           justify-content:center; gap:24px; padding:24px; box-sizing:border-box; }}
    img {{ max-width:min(92vw,480px); height:auto; border-radius:24px; }}
    a.btn {{ background:#f472b6; color:#1b1f2a; font-weight:700; text-decoration:none;
             padding:14px 28px; border-radius:12px; }}
  </style>
</head>
<body>
  <img src="{card_url}" alt="{title}"/>
  <a class="btn" href="{base}/?w={token}">Convert your own lift &rarr;</a>
</body>
</html>"##,
        title = esc(&title),
        description = esc(&description),
        token = esc(&token),
    );

    Ok(Html(html).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn ua(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("user-agent", HeaderValue::from_str(value).unwrap());
        h
    }

    #[test]
    fn crawlers_are_detected() {
        assert!(is_crawler(&ua("Mozilla/5.0 (compatible; Discordbot/2.0)")));
        assert!(is_crawler(&ua("Twitterbot/1.0")));
        assert!(is_crawler(&ua("facebookexternalhit/1.1")));
        assert!(!is_crawler(&ua(
            "Mozilla/5.0 (Macintosh) AppleWebKit/537.36 Chrome/120 Safari/537.36"
        )));
        assert!(!is_crawler(&HeaderMap::new()));
    }

    #[test]
    fn spec_round_trips() {
        for (amt, unit, token) in [
            (225.0, InputWeightType::Lbs, "225lbs"),
            (100.5, InputWeightType::Kgs, "100.5kgs"),
        ] {
            assert_eq!(spec_token(amt, &unit), token);
            let (a, u) = parse_weight_spec(token).unwrap();
            assert!((a - amt).abs() < f64::EPSILON);
            assert_eq!(u.to_string(), unit.to_string());
        }
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_weight_spec("abc").is_none());
        assert!(parse_weight_spec("0lbs").is_none());
        assert!(parse_weight_spec("-5kg").is_none());
        assert!(parse_weight_spec("10furlongs").is_none());
    }
}
