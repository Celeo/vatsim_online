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

use api::Vatsim;

mod api;
mod models;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("vatsim_online", log::LevelFilter::Debug)
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() {
    setup_logger().expect("Could not configure logger");
    let vatsim = Vatsim::new().expect("Could not set up access to VATSim API");
    let data = vatsim.get_data().expect("Could not get VATSim data");
    // TODO
}
