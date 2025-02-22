use std::{
    fmt::{Debug, Error, Formatter},
    ops::{Deref, DerefMut},
};

use image::{DynamicImage, GenericImageView};
use libwebp_sys::{
    VP8StatusCode, WebPConfig, WebPEncodingError, WebPFree, WebPMemoryWrite, WebPMemoryWriterInit,
    WebPPicture, WebPPictureFree, WebPPictureImportRGB, WebPValidateConfig,
};

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
