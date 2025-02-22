use clap::Parser;
use logging::Logging;
use std::path::PathBuf;

mod args;
mod file_utils;
mod format_utils;
mod image_processing;
mod logging;
mod webp_wrapper;

struct Depth {
    current: u16,
    max: u16,
}

fn main() {
    let result = try_main();
    if result.is_err() {
        eprintln!("{:#?}", result.err());
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Cli::try_parse().unwrap();

    let output_path: PathBuf = args.output_path().map_err(|error| error).unwrap();
    let input_path: PathBuf = args.input_path().map_err(|error| error).unwrap();
    let config = image_processing::generate_config(&args);

    let logging = Logging::start();

    let paths = file_utils::Paths::build(input_path, output_path, args.max_depth);
    let (input_size, output_size, count) =
        image_processing::convert_file_all(paths, &config, args.use_initial_if_smaller);

    logging.end(input_size, output_size, count);

    Ok(())
}
