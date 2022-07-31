#![deny(
    clippy::all,
    clippy::pedantic,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

mod api;
mod interface;
mod models;

use anyhow::Result;
use api::Vatsim;
use clap::Parser;

const LOG_FILE_NAME: &str = "vatsim_online.log";

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Enable debug logging to a 'vatsim_online.log' file
    #[clap(short, long)]
    debug: bool,
}

/// Configure the debug file logger.
fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ));
        })
        .level(log::LevelFilter::Info)
        .level_for("vatsim_online", log::LevelFilter::Debug)
        .chain(fern::log_file(LOG_FILE_NAME)?)
        .apply()?;
    Ok(())
}

/// Entry point.
fn main() {
    let args = Args::parse();
    if args.debug {
        setup_logger().expect("Could not configure logger");
    }
    let vatsim = Vatsim::new().expect("Could not set up access to VATSIM API");
    let data = vatsim.get_data().expect("Could not get VATSIM data");
    interface::run(data).expect("Could not set up interface");
}
