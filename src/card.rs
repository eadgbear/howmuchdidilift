//! Server-side share-card rendering: composes an SVG (lift number + Noto emoji
//! + "= N objects") and rasterizes it to PNG via `resvg`. Used for the social
//! `og:image` so results unfurl in Discord/iMessage/Twitter.

use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::Rng;
use resvg::{tiny_skia, usvg};

/// Card dimensions — a tight near-square so content fills the frame instead of
/// leaving wide banner margins.
const WIDTH: u32 = 640;
const HEIGHT: u32 = 640;

/// Card pixel dimensions `(width, height)` — used for `og:image:width/height`.
#[must_use]
pub const fn dimensions() -> (u32, u32) {
    (WIDTH, HEIGHT)
}

/// Root dir holding bundled `fonts/` and `emoji/` assets. Relative to CWD, which
/// is the repo root in dev and `/usr/app` in the container (see dockerfile COPY).
fn asset_root() -> PathBuf {
    PathBuf::from("assets")
}

/// XML-escape text destined for an SVG `<text>` node.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const SKIN_TONES: [&str; 5] = ["1f3fb", "1f3fc", "1f3fd", "1f3fe", "1f3ff"];

/// If `base` is a skin-tone-capable emoji (a toned variant file exists) and
/// doesn't already carry a tone, pick a random one — so hands/people/body-parts
/// vary each render. Otherwise return it unchanged.
fn randomize_skin_tone(base: &str) -> String {
    if SKIN_TONES.iter().any(|t| base.ends_with(t)) {
        return base.to_string();
    }
    let probe = asset_root()
        .join("emoji")
        .join(format!("emoji_u{base}_1f3fb.svg"));
    if probe.exists() {
        let t = SKIN_TONES[rand::thread_rng().gen_range(0..SKIN_TONES.len())];
        return format!("{base}_{t}");
    }
    base.to_string()
}

/// Read a single Noto emoji SVG for a codepoint token (e.g. "1f35a" or the
/// two-part "1f1e9_1f1ea" of a flag) and base64-encode it. `None` if missing.
/// Skin-tone-capable emoji get a random tone.
fn emoji_b64(token: &str) -> Option<String> {
    let cp = token.trim().to_lowercase();
    // Guard against path traversal — codepoints are hex + underscore only.
    if cp.is_empty() || !cp.chars().all(|c| c.is_ascii_hexdigit() || c == '_') {
        return None;
    }
    let cp = randomize_skin_tone(&cp);
    let path = asset_root().join("emoji").join(format!("emoji_u{cp}.svg"));
    std::fs::read(path).ok().map(|b| STANDARD.encode(b))
}

fn image_el(x: i32, y: i32, size: i32, b64: &str) -> String {
    format!(
        r#"<image x="{x}" y="{y}" width="{size}" height="{size}" href="data:image/svg+xml;base64,{b64}"/>"#
    )
}

/// Pick a font size that keeps `text` within the card width, capped at `base`.
/// `k` approximates average glyph-width-to-height for Oxanium (wide face).
fn fit_font(text: &str, base: f64, k: f64) -> i32 {
    let usable = f64::from(WIDTH - 72);
    let n = text.chars().count().max(1) as f64;
    (usable / (k * n)).min(base).round().max(16.0) as i32
}

/// Edge length of a rendered emoji (same whether one or two are shown).
const EMOJI_SIZE: i32 = 230;

/// Render the icon band from an icon spec of up to two `+`-joined emoji tokens
/// (e.g. "1f1e9_1f1ea+1f697" = flag + car), vertically centered on `center_y`.
/// One emoji is centered; two sit side by side, aligned on the same centerline.
/// A missing/empty spec falls back to a question mark.
fn emoji_images(spec: Option<&str>, center_y: i32) -> String {
    let mut b64s: Vec<String> = spec
        .unwrap_or("")
        .split('+')
        .filter(|t| !t.trim().is_empty())
        .take(2)
        .filter_map(emoji_b64)
        .collect();

    // No usable emoji → fall back to a question mark so the band isn't empty.
    if b64s.is_empty() {
        b64s = emoji_b64("2753").into_iter().collect();
    }

    let size = EMOJI_SIZE;
    let top = center_y - size / 2;
    match b64s.as_slice() {
        [] => String::new(),
        [one] => image_el((WIDTH as i32 - size) / 2, top, size, one),
        [a, b, ..] => {
            let gap = 24;
            let total = size * 2 + gap;
            let x0 = (WIDTH as i32 - total) / 2;
            image_el(x0, top, size, a) + &image_el(x0 + size + gap, top, size, b)
        }
    }
}

/// Uppercase for the all-caps cyberpunk aesthetic.
fn up(s: &str) -> String {
    s.to_uppercase()
}

/// Build the composed SVG markup for a card. Four major rows — primary weight,
/// secondary weight, emoji, measurement — are evenly spaced; a small label
/// ("YOU LIFTED", "OR") hugs above each weight with an equal gap, and a second
/// "OR" connector sits between the secondary weight and the emoji.
fn build_svg(
    primary: &str,
    secondary: &str,
    count: &str,
    units: &str,
    permalink: &str,
    icon: Option<&str>,
) -> String {
    // All-caps display strings.
    let primary = up(primary);
    let secondary = up(secondary);
    let units = up(units);
    let permalink = up(permalink);

    // Oxanium is wide — size each line to fit the card width.
    let f_primary = fit_font(&primary, 56.0, 0.78);
    let f_secondary = fit_font(&secondary, 34.0, 0.74);
    let f_measure = fit_font(count, 56.0, 0.74);
    let f_units = fit_font(&units, 48.0, 0.74);
    let f_url = fit_font(&permalink, 22.0, 0.60);

    // Four "major" rows — primary weight, secondary weight, emoji, measurement —
    // are evenly spaced. A small label ("YOU LIFTED", "OR") hugs above each weight
    // with the SAME visual gap. `half` approximates a centered line's half-height.
    // ─── LAYOUT KNOBS (tweak these, then `cargo run --example gencards`) ───
    // FS_LABEL / FS_OR : font size of the "YOU LIFTED" and "OR" mini-labels.
    // GAP              : vertical gap between a mini-label and the weight it hugs.
    // m1 / m4          : center-y of the top (primary) and bottom (measurement)
    //                    major rows; the other two majors are evenly spaced between.
    // EMOJI_SIZE       : emoji edge length (const near top of file).
    // y_url            : footer/permalink center-y (raise to add bottom padding).
    // font sizes       : the `fit_font(text, BASE, k)` BASE caps per line below.
    const FS_LABEL: f64 = 28.0;
    const FS_OR: f64 = 24.0;
    const GAP: f64 = 20.0;
    let emoji_half = f64::from(EMOJI_SIZE) / 2.0;
    let half = |fs: f64| fs * 0.36;

    let you_lifted_label = GAP + FS_LABEL;
    let y_primary_label = you_lifted_label + 2.0 * GAP + half(FS_LABEL);
    //let y_or1 = (m2 - half(f_secondary.into()) - GAP - half(FS_OR)).round() as i32;
    let y_or1 = y_primary_label + 0.8 * GAP + FS_OR;
    let y_secondary_label = y_or1 + 0.6 * GAP + FS_OR;
    // Second "or" is a connector: centered between the secondary weight and emoji.
    let y_or2 = y_secondary_label + 0.6 * GAP + FS_OR;
    let emoji_center: f64 = y_or2 + GAP + emoji_half;
    let measure_label: f64 = emoji_center + emoji_half + 1.5 * GAP;
    let unit_label: f64 = measure_label + 1.5 * GAP + FS_OR;
    let y_url = f64::from(HEIGHT) - 2.0 * GAP;

    let icon_el = emoji_images(icon, emoji_center.round() as i32);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}">
  <rect width="{WIDTH}" height="{HEIGHT}" fill="#1b1f2a"/>
  <rect x="20" y="20" width="{bw}" height="{bh}" rx="24" fill="none" stroke="#2a3040" stroke-width="4"/>
  <text x="{cx}" y="{you_lifted_label}" font-family="Oxanium" font-size="28" fill="#8b93a7" text-anchor="middle" dominant-baseline="central">YOU LIFTED</text>
  <text x="{cx}" y="{y_primary_label}" font-family="Oxanium" font-weight="bold" font-size="{f_primary}" fill="#ffffff" text-anchor="middle" dominant-baseline="central">{primary}</text>
  <text x="{cx}" y="{y_or1}" font-family="Oxanium" font-size="24" fill="#8b93a7" text-anchor="middle" dominant-baseline="central">OR</text>
  <text x="{cx}" y="{y_secondary_label}" font-family="Oxanium" font-size="{f_secondary}" fill="#c3c9d6" text-anchor="middle" dominant-baseline="central">{secondary}</text>
  <text x="{cx}" y="{y_or2}" font-family="Oxanium" font-size="24" fill="#8b93a7" text-anchor="middle" dominant-baseline="central">OR</text>
  {icon_el}
  <text x="{cx}" y="{measure_label}" font-family="Oxanium" font-weight="bold" font-size="{f_measure}" fill="#ffffff" text-anchor="middle" dominant-baseline="central"><tspan fill="#f472b6">{count}</tspan></text>
  <text x="{cx}" y="{unit_label}" font-family="Oxanium" font-weight="bold" font-size="{f_units}" fill="#ffffff" text-anchor="middle" dominant-baseline="central">{units}</text>
  <text x="{cx}" y="{y_url}" font-family="Oxanium" font-size="{f_url}" fill="#5c6478" text-anchor="middle" dominant-baseline="central">{permalink}</text>
</svg>"##,
        bw = WIDTH - 40,
        bh = HEIGHT - 40,
        cx = WIDTH / 2,
        count = esc(count),
        units = esc(&units),
        primary = esc(&primary),
        secondary = esc(&secondary),
        permalink = esc(&permalink),
    )
}

/// Render a share card to PNG bytes.
pub fn render_card(
    primary: &str,
    secondary: &str,
    count: &str,
    units: &str,
    permalink: &str,
    icon: Option<&str>,
) -> Result<Vec<u8>, String> {
    let svg = build_svg(primary, secondary, count, units, permalink, icon);

    let font_path = asset_root().join("fonts").join("Oxanium.ttf");
    let font_bytes =
        std::fs::read(&font_path).map_err(|e| format!("failed to read {font_path:?}: {e}"))?;

    let mut opt = usvg::Options::default();
    opt.font_family = "Oxanium".to_string();
    opt.fontdb_mut().load_font_data(font_bytes);

    let tree = usvg::Tree::from_str(&svg, &opt).map_err(|e| e.to_string())?;

    let mut pixmap = tiny_skia::Pixmap::new(WIDTH, HEIGHT).ok_or("failed to alloc pixmap")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap.encode_png().map_err(|e| e.to_string())
}

/// Whether the bundled font asset exists (used to fail fast / skip in tests).
pub fn assets_available() -> bool {
    asset_root().join("fonts").join("Oxanium.ttf").exists() && Path::new("assets/emoji").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_card_png() {
        let png = render_card(
            "1,234 lbs",
            "559.7 kgs",
            "74.63",
            "Bags of Rice",
            "howmuchdidilift.com/w/1234lbs",
            Some("1f35a"),
        )
        .expect("render should succeed");
        assert!(png.len() > 1000, "png too small: {}", png.len());
        assert_eq!(&png[1..4], b"PNG", "not a png");
    }
}
