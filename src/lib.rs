// The CI runs clippy with `-W clippy::pedantic -W clippy::nursery -D warnings`.
// These opinionated groups fire heavily on loco-generated code (handlers without
// `# Errors` docs, prelude wildcard imports, etc.); allow the noisy ones crate-wide
// so real lints still surface.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::doc_lazy_continuation,
    clippy::empty_line_after_doc_comments,
    clippy::field_reassign_with_default,
    clippy::implicit_clone,
    clippy::items_after_statements,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::redundant_clone,
    clippy::suboptimal_flops,
    clippy::unnecessary_debug_formatting,
    clippy::unnecessary_mut_passed,
    clippy::use_self,
    clippy::wildcard_imports
)]

pub mod app;
pub mod card;
pub mod controllers;
pub mod fact_store;
pub mod mailers;
pub mod measures_config;
pub mod models;
pub mod tasks;
pub mod views;
pub mod workers;
