use std::{fs, path::PathBuf};

use image::DynamicImage;
use libwebp_sys::WebPConfig;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{args, file_utils, logging::Logging, webp_wrapper};

pub fn generate_config(args: &args::Cli) -> WebPConfig {
    let mut config: WebPConfig = WebPConfig::new().unwrap();
    config.lossless = if args.quality == 100 {
        args.lossless
    } else {
        0
    } as i32;
    config.quality = args.quality as f32;
    config.method = args.method as i32;
    // Multi threading
    config.thread_level = 1;

    config
}

pub fn convert_file_all(
    paths: file_utils::Paths,
    config: &WebPConfig,
    use_initial_if_smaller: u8,
) -> (u64, u64, u64) {
    let images = paths.input.images;
    let input_root = paths.input.root;
    let output_root = paths.output_root;
    images
        .iter()
        .par_bridge()
        .map(|path| {
            let output_path = if path.starts_with(&input_root) {
                let stripped_path = path.strip_prefix(&input_root).unwrap();
                &output_root.join(stripped_path)
            } else {
                &output_root
            };

            let converted_file = convert_file(path, output_path, config, use_initial_if_smaller);
            if converted_file.is_err() {
                eprintln!("{:?}", converted_file.err());
                return (path.metadata().unwrap().len(), 0, 1);
            }
            return (path.metadata().unwrap().len(), converted_file.unwrap(), 1);
        })
        .reduce(
            || (0, 0, 0),
            |(input_size_0, output_size_0, count_0), (input_size_1, output_size_1, count_1)| {
                (
                    input_size_0 + input_size_1,
                    output_size_0 + output_size_1,
                    count_0 + count_1,
                )
            },
        )
}

/// Returns new file size
fn convert_file(
    input: &PathBuf,
    output: &PathBuf,
    config: &WebPConfig,
    use_initial_if_smaller: u8,
) -> Result<u64, Box<dyn std::error::Error>> {
    let logging = Logging::start();

    let file_name = &input
        .file_stem()
        .ok_or_else(|| format!("The file name: {:?} does not exist!", &input.file_name()))?
        .to_string_lossy()
        .to_string();

    let img = open_image_from_path(input.clone());
    if img.is_none() {
        Err(format!("{:?} is not an image", &input.file_name().unwrap()))?
    }
    let img = img.unwrap();

    let result = webp_wrapper::image_to_webp(img.clone(), &config);
    let webp = result.map_err(|_| "Failed to convert image")?;

    let output_path = if !(&output).exists() {
        if output.extension().is_some() {
            fs::create_dir_all(&output.parent().unwrap())?;
        } else {
            fs::create_dir_all(&output)?;
        }

        output
            .parent()
            .unwrap()
            .join(file_name)
            .with_extension("webp")
    } else {
        output.join(file_name).with_extension("webp")
    };

    let input_size = input.metadata().unwrap().len();
    let mut output_size = webp.len() as u64;

    if use_initial_if_smaller == 1 && input_size < output_size {
        output_size = input_size;
        fs::write(&output_path, img.into_bytes()).unwrap();
    } else {
        fs::write(&output_path, &*webp).unwrap();
    }

    logging.log_row(
        input.file_name().unwrap().to_string_lossy().to_string(),
        input_size,
        output_size,
    );

    Ok(output_size as u64)
}

fn open_image_from_path(path: PathBuf) -> Option<DynamicImage> {
    match image::open(path) {
        Ok(img) => {
            return Some(img);
        }
        Err(_) => {
            return None;
        }
    }
}
