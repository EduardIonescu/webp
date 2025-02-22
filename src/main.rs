use clap::Parser;
use image::{DynamicImage, GenericImageView};
use libwebp_sys::{
    VP8StatusCode, WebPConfig, WebPEncodingError, WebPFree, WebPMemoryWrite, WebPMemoryWriterInit,
    WebPPicture, WebPPictureFree, WebPPictureImportRGB, WebPValidateConfig,
};
use rayon::prelude::*;
use std::{
    env,
    fmt::{Debug, Error, Formatter},
    fs,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Parser)]
struct Cli {
    /// Input path
    input: PathBuf,

    /// Output path, uses root if not provided
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Quality from 0 to 100
    #[arg(short, long, default_value_t = 100)]
    quality: u8,

    #[arg(short, long, default_value_t = 1)]
    lossless: u8,

    #[arg(short, long, default_value_t = 6)]
    method: u8,

    #[arg(long, default_value_t = 8)]
    max_depth: u16,

    #[arg(long, default_value_t = 0)]
    use_initial_if_smaller: u8,
}

impl Cli {
    fn input_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if self.input.try_exists().is_err() {
            Err(format!(
                "The path: {} does not exist!",
                &self.input.to_str().unwrap()
            ))?
        }

        if !self.input.is_file() && !self.input.is_dir() {
            Err(format!(
                "The path: {} does not exist!",
                self.input.to_str().unwrap()
            ))?
        }

        Ok(self.input.clone())
    }

    fn output_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_dir = match &self.output {
            Some(path) => {
                if path.is_absolute() {
                    path.clone()
                } else {
                    env::current_dir()?.join(path)
                }
            }
            None => self
                .input
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
        };

        Ok(output_dir)
    }
}

struct Depth {
    current: u16,
    max: u16,
}

struct Paths {
    input: InputPaths,
    output_root: PathBuf,
}
struct InputPaths {
    root: PathBuf,
    images: Vec<PathBuf>,
}

impl Paths {
    pub fn build(input_path: PathBuf, output_path: PathBuf, max_depth: u16) -> Paths {
        let depth = Depth {
            current: 0,
            max: max_depth,
        };

        let mut all_files: Vec<PathBuf> = Vec::new();
        Self::flatten_dir(input_path.clone(), &mut all_files, depth);

        Self {
            input: InputPaths {
                root: input_path,
                images: all_files,
            },
            output_root: output_path,
        }
    }

    /// Returns (input_size, output_size)
    fn flatten_dir(input_path: PathBuf, all_files: &mut Vec<PathBuf>, depth: Depth) {
        if input_path.is_file() {
            all_files.push(input_path.clone());
            return;
        }
        if input_path.is_dir() {
            for path in input_path.read_dir().unwrap() {
                if path.is_err() || depth.current + 1 > depth.max {
                    return;
                }

                let new_depth = Depth {
                    current: depth.current + 1,
                    max: depth.max,
                };
                Self::flatten_dir(path.unwrap().path(), all_files, new_depth);
            }
        }
    }
}

fn main() {
    let result = try_main();
    if result.is_err() {
        eprintln!("{:#?}", result.err());
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let output_path: PathBuf = args.output_path().map_err(|error| error).unwrap();
    let input_path: PathBuf = args.input_path().map_err(|error| error).unwrap();
    let config = generate_config(&args);

    let now = Instant::now();

    println!(
        "{0:<30} | {1:<10} | {2:<10} | {3:<10}",
        "Name", "Input", "Output", "Duration"
    );

    let paths = Paths::build(input_path, output_path, args.max_depth);
    let (input_size, output_size, count) =
        convert_file_all(paths, &config, args.use_initial_if_smaller);

    println!("\n--- TOTAL --- ");
    println!(
        "{0:<12} | {1:<12} | {2:<12} | {3:<12} | {4:<12}",
        "Input Size", "Output Size", "Reduction", "Duration", "Images Count"
    );
    let reduction_difference = input_size as f64 - output_size as f64;
    let reduction_percentage = 100.0 * reduction_difference / input_size as f64;
    println!(
        "{0:<12} | {1:<12} | {2:<12} | {3:<12} | {4:<12}",
        format_size(input_size),
        format_size(output_size),
        format!("{:.1?} %", reduction_percentage),
        format_millis(now.elapsed().as_millis()),
        count
    );

    Ok(())
}

const GB: u64 = 2_u64.pow(30);
const MB: u64 = 2_u64.pow(20);
const KB: u64 = 2_u64.pow(10);
fn format_size(size: u64) -> String {
    if size > GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size > MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size > KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{:.2} B", size)
    }
}

fn generate_config(args: &Cli) -> WebPConfig {
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

fn convert_file_all(
    paths: Paths,
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
    let now = Instant::now();

    println!("{:?}", input);
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

    let result = image_to_webp(img.clone(), &config);
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
    let elapsed_time = now.elapsed();

    println!(
        "{0:<30} | {1:<10} | {2:<10} | {3:<10}",
        input.file_name().unwrap().to_string_lossy(),
        format_size(input_size),
        format_size(output_size),
        format_millis(elapsed_time.as_millis())
    );

    Ok(output_size as u64)
}

fn format_millis(ms: u128) -> String {
    if ms < 1000 {
        return format!("{} ms", ms);
    }

    let seconds = ms as f64 / 1000.0;

    if seconds < 60.0 {
        return format!("{:.1} s", seconds);
    }

    return format!("{} min {:.1} s", (seconds / 60.0).floor(), seconds % 60.0);
}

pub fn open_image_from_path(path: PathBuf) -> Option<DynamicImage> {
    match image::open(path) {
        Ok(img) => {
            return Some(img);
        }
        Err(_) => {
            return None;
        }
    }
}

pub fn image_to_webp(
    img: DynamicImage,
    config: &WebPConfig,
) -> Result<WebPMemory, WebPEncodingError> {
    let (width, height) = img.dimensions();
    let img = img.into_rgb8();

    unsafe {
        let mut picture = new_picture(&img, width, height);
        let result = encode(&mut picture, config);
        result
    }
}

/// This struct represents a safe wrapper around memory owned by libwebp.
/// Its data contents can be accessed through the Deref and DerefMut traits.
pub struct WebPMemory(pub(crate) *mut u8, pub usize);

impl Debug for WebPMemory {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("WebpMemory").finish()
    }
}

impl Drop for WebPMemory {
    fn drop(&mut self) {
        unsafe { WebPFree(self.0 as _) }
    }
}

impl Deref for WebPMemory {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.0, self.1) }
    }
}

impl DerefMut for WebPMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.0, self.1) }
    }
}

unsafe fn encode(
    picture: &mut WebPPicture,
    config: &WebPConfig,
) -> Result<WebPMemory, WebPEncodingError> {
    if WebPValidateConfig(config) == 0 {
        return Err(WebPEncodingError::VP8_ENC_ERROR_INVALID_CONFIGURATION);
    }
    let mut ww = std::mem::MaybeUninit::uninit();
    WebPMemoryWriterInit(ww.as_mut_ptr());
    picture.writer = Some(WebPMemoryWrite);
    picture.custom_ptr = ww.as_mut_ptr() as *mut std::ffi::c_void;
    let status = libwebp_sys::WebPEncode(config, picture);
    let ww = ww.assume_init();
    let mem = WebPMemory(ww.mem, ww.size as usize);
    if status != VP8StatusCode::VP8_STATUS_OK as i32 {
        Ok(mem)
    } else {
        Err(picture.error_code)
    }
}

#[derive(Debug)]
pub struct ManagedPicture(pub WebPPicture);

impl Drop for ManagedPicture {
    fn drop(&mut self) {
        unsafe { WebPPictureFree(&mut self.0 as _) }
    }
}

impl Deref for ManagedPicture {
    type Target = WebPPicture;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ManagedPicture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub unsafe fn new_picture(image: &[u8], width: u32, height: u32) -> ManagedPicture {
    let mut picture = WebPPicture::new().unwrap();
    picture.use_argb = 1;
    picture.width = width as i32;
    picture.height = height as i32;
    WebPPictureImportRGB(&mut picture, image.as_ptr(), width as i32 * 3);
    ManagedPicture(picture)
}
