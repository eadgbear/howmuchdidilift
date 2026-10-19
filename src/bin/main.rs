// See src/lib.rs for why these pedantic/nursery lints are allowed crate-wide.
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

use liftcalc::app::App;
use loco_rs::cli;
use migration::Migrator;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    cli::main::<App, Migrator>().await
}
