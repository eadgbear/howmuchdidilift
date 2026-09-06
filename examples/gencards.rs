//! Render sample share-cards to /tmp so you can eyeball layout changes fast.
//!
//!   cargo run --example gencards      # writes /tmp/card_*.png
//!
//! Layout knobs live in src/card.rs::build_svg. Count formatting (no scientific
//! notation) lives in src/controllers/measure.rs::fmt_count — mirrored here.

fn fmt_count(v: f64) -> String {
    if v >= 1.0 {
        return format!("{v:.2}");
    }
    let places = usize::try_from(((-v.log10()).ceil() as i32 + 3).clamp(2, 24)).unwrap_or(2);
    format!("{v:.places$}")
}

fn main() {
    let lifted = 225.0 * 453.592; // 225 lb in grams
                                  // (grams-per-one, units, icon spec)
    let samples = [
        (100.0, "Slices of Pizza", "1f355", "pizza"),
        (
            5.357e17,
            "Germanys (total landmass)",
            "1f1e9_1f1ea",
            "germany_land",
        ),
        (
            8.635e13,
            "Chinas (population mass)",
            "1f1e8_1f1f3+1f9d1",
            "china_pop",
        ),
        (8.1e17, "Mount Everests", "1f3d4", "everest"),
        (5.972e27, "Planet Earths", "1f30d", "earth"),
    ];
    for (grams, units, icon, slug) in samples {
        let count = fmt_count(lifted / grams);
        let png = liftcalc::card::render_card(
            "225 lbs",
            "102.06 kgs",
            &count,
            units,
            "howmuchdidilift.com/w/225lbs",
            Some(icon),
        )
        .expect("render");
        let out = format!("/tmp/card_{slug}.png");
        std::fs::write(&out, png).expect("write");
        println!("{out}: {count} {units}");
    }
}
