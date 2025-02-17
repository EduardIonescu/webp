use clap::Parser;
use image::{DynamicImage, GenericImageView};
use libwebp_sys::{
    VP8StatusCode, WebPConfig, WebPEncodingError, WebPFree, WebPMemoryWrite, WebPMemoryWriterInit,
    WebPPicture, WebPPictureFree, WebPPictureImportRGB, WebPValidateConfig,
};
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
}

impl Cli {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let output_path = args
        .output_path()
        .map_err(|_| "Output file is invalid")
        .unwrap();

    if args.input.try_exists().is_err() {
        return Err(format!(
            "The path: {} does not exist!",
            args.input.to_str().unwrap()
        ))?;
    }

    if !args.input.is_file() && !args.input.is_dir() {
        Err(format!(
            "The path: {} does not exist!",
            args.input.to_str().unwrap()
        ))?
    }

    let mut config: WebPConfig = WebPConfig::new().unwrap();
    config.lossless = args.lossless as i32;
    config.quality = args.quality as f32;
    config.method = 5;
    // Multi threading
    config.thread_level = 1;

    let now = Instant::now();

    let _ = convert_file(args.input, output_path, config);

    let elapsed_time = now.elapsed();
    println!(
        "Running slow_function() took {} seconds.",
        elapsed_time.as_millis()
    );

    Ok(())
}

fn convert_file(
    input: PathBuf,
    output: PathBuf,
    config: WebPConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = &input
        .file_stem()
        .ok_or_else(|| format!("The file name: {:?} does not exist!", &input.file_name()))?
        .to_string_lossy()
        .to_string();

    let img = open_image_from_path(input).unwrap();

    let result = image_to_webp(img, &config);
    let webp = result.map_err(|_| "Failed to convert image")?;

    let output_path = output.join(file_name).with_extension("webp");

    fs::write(&output_path, &*webp).unwrap();

    Ok(())
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
