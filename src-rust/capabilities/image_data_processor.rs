use crate::contract::ImageProcessorProtocol;
use crate::taxonomy::engine_name_vo::EngineNameVo;
use anyhow::anyhow;
use fast_image_resize as fr;
use image::{DynamicImage, ImageBuffer};
use rayon::prelude::*;
use std::num::NonZeroU32;

/// ONNX/Compute model input dimensions (BRIA RMBG-2.0 requires 1024x1024).
const ML_MODEL_IMAGE_WIDTH: u32 = 1024;
const ML_MODEL_IMAGE_HEIGHT: u32 = 1024;

/// ImageNet normalization mean & std (RMBG-2.0 expects ImageNet stats).
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Edge sharpening thresholds for alpha mask refinement.
const EDGE_LOW_THRESHOLD: f32 = 0.15;
const EDGE_HIGH_THRESHOLD: f32 = 0.85;

/// Image processor that handles preprocessing and postprocessing for ML inference.
pub struct ImageDataProcessor {
    engine_name: EngineNameVo,
}

impl ImageDataProcessor {
    pub fn new() -> Self {
        Self {
            engine_name: EngineNameVo::new("ImageDataProcessor"),
        }
    }
}

impl Default for ImageDataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageProcessorProtocol for ImageDataProcessor {
    fn processor_preprocess(&self, image: &DynamicImage) -> anyhow::Result<Vec<f32>> {
        preprocess_image(image)
    }

    fn processor_postprocess(&self, model_result: &[f32]) -> anyhow::Result<DynamicImage> {
        postprocess_image(model_result)
    }

    fn processor_apply_mask(&self, original: &DynamicImage, mask: &DynamicImage) -> DynamicImage {
        apply_mask(original, mask)
    }

    fn processor_resize(
        &self,
        image: &DynamicImage,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Vec<u8>> {
        resize_rgba(image, width, height)
    }

    fn processor_engine_name(&self) -> EngineNameVo {
        self.engine_name.clone()
    }
}

/// Preprocesses image by resizing to 1024x1024 and creating normalized matrix buffer.
fn resize_rgba(
    img: &DynamicImage,
    target_width: u32,
    target_height: u32,
) -> anyhow::Result<Vec<u8>> {
    let width = NonZeroU32::new(img.width()).ok_or(anyhow!("Invalid image width"))?;
    let height = NonZeroU32::new(img.height()).ok_or(anyhow!("Invalid image height"))?;
    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )?;

    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut())?;

    let dst_width = NonZeroU32::new(target_width).ok_or(anyhow!("Invalid target width"))?;
    let dst_height = NonZeroU32::new(target_height).ok_or(anyhow!("Invalid target height"))?;
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    let mut dst_view = dst_image.view_mut();
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Bilinear));
    resizer.resize(&src_image.view(), &mut dst_view)?;

    alpha_mul_div.divide_alpha_inplace(&mut dst_view)?;
    Ok(dst_image.into_vec())
}

/// Preprocesses DynamicImage into a flattened float array tensor.
fn preprocess_image(image: &DynamicImage) -> anyhow::Result<Vec<f32>> {
    let img_vec = resize_rgba(image, ML_MODEL_IMAGE_WIDTH, ML_MODEL_IMAGE_HEIGHT)?;
    let size = (ML_MODEL_IMAGE_WIDTH * ML_MODEL_IMAGE_HEIGHT) as usize;

    let mut tensor = vec![0.0f32; size * 3];
    let inv_mean = [
        1.0 / 255.0 / IMAGENET_STD[0],
        1.0 / 255.0 / IMAGENET_STD[1],
        1.0 / 255.0 / IMAGENET_STD[2],
    ];
    let offsets = [
        -IMAGENET_MEAN[0] / IMAGENET_STD[0],
        -IMAGENET_MEAN[1] / IMAGENET_STD[1],
        -IMAGENET_MEAN[2] / IMAGENET_STD[2],
    ];

    tensor
        .par_chunks_mut(size)
        .enumerate()
        .for_each(|(c, chunk)| {
            for (i, pixel) in chunk.iter_mut().enumerate().take(size) {
                let idx = i * 4;
                *pixel = img_vec[idx + c] as f32 * inv_mean[c] + offsets[c];
            }
        });

    Ok(tensor)
}

/// Postprocesses raw inference output into a grayscale mask DynamicImage.
fn postprocess_image(model_result: &[f32]) -> anyhow::Result<DynamicImage> {
    let size = (ML_MODEL_IMAGE_WIDTH * ML_MODEL_IMAGE_HEIGHT) as usize;

    let mut sigmoid = vec![0.0f32; size];
    sigmoid
        .par_iter_mut()
        .zip(model_result[..size].par_iter())
        .for_each(|(out, &v)| {
            *out = 1.0 / (1.0 + (-v).exp());
        });

    let (mi, ma) = sigmoid
        .par_iter()
        .fold(
            || (f32::INFINITY, f32::NEG_INFINITY),
            |(mi, ma), &s| (mi.min(s), ma.max(s)),
        )
        .reduce(
            || (f32::INFINITY, f32::NEG_INFINITY),
            |(mi1, ma1), (mi2, ma2)| (mi1.min(mi2), ma1.max(ma2)),
        );

    let range = if ma > mi { ma - mi } else { 1.0 };
    let inv_range = 255.0 / range;

    let mut raw = vec![0u8; size * 3];
    raw.par_chunks_mut(3)
        .zip(sigmoid.par_iter())
        .for_each(|(chunk, &s)| {
            let val = ((s - mi) * inv_range) as u8;
            chunk[0] = val;
            chunk[1] = val;
            chunk[2] = val;
        });

    let img = ImageBuffer::from_raw(ML_MODEL_IMAGE_WIDTH, ML_MODEL_IMAGE_HEIGHT, raw)
        .ok_or(anyhow!("Failed to create postprocess result image"))?;
    Ok(DynamicImage::ImageRgb8(img))
}

/// Refines alpha mask using edge-sharpening and blends with original image.
fn apply_mask(original_image: &DynamicImage, mask_image: &DynamicImage) -> DynamicImage {
    let width = mask_image.width();
    let height = mask_image.height();
    let size = (width * height) as usize;

    let orig = original_image.to_rgba8().into_raw();
    let mask = mask_image.to_rgb8().into_raw();

    let mut out = vec![0u8; size * 4];
    let inv_255 = 1.0 / 255.0;
    let edge_range_inv = 1.0 / (EDGE_HIGH_THRESHOLD - EDGE_LOW_THRESHOLD);

    out.par_chunks_mut(4).enumerate().for_each(|(i, chunk)| {
        let r = orig[i * 4];
        let g = orig[i * 4 + 1];
        let b = orig[i * 4 + 2];
        let raw_alpha_val = mask[i * 3] as f32 * inv_255;

        let sharpened = if raw_alpha_val < EDGE_LOW_THRESHOLD {
            0.0
        } else if raw_alpha_val > EDGE_HIGH_THRESHOLD {
            1.0
        } else {
            (raw_alpha_val - EDGE_LOW_THRESHOLD) * edge_range_inv
        };

        let smooth = sharpened * sharpened * (3.0 - 2.0 * sharpened);
        let alpha = (smooth * 255.0).round() as u8;

        if alpha == 0 {
            chunk[0] = 0;
            chunk[1] = 0;
            chunk[2] = 0;
            chunk[3] = 0;
        } else if raw_alpha_val > 0.01 {
            let scale = 1.0 / raw_alpha_val;
            chunk[0] = ((r as f32 * scale).round() as u32).min(255) as u8;
            chunk[1] = ((g as f32 * scale).round() as u32).min(255) as u8;
            chunk[2] = ((b as f32 * scale).round() as u32).min(255) as u8;
            chunk[3] = alpha;
        } else {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = alpha;
        }
    });

    let img =
        ImageBuffer::from_raw(width, height, out).expect("Failed to create mask output image");
    DynamicImage::ImageRgba8(img)
}
