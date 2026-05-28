use std::path::Path;
use std::sync::{Arc, Mutex};
use std::num::NonZeroU32;
use anyhow::{anyhow, Context};
use colored::Colorize;
use fast_image_resize as fr;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, Rgba, RgbaImage};
use ndarray::{s, Array3, Axis};
use crate::contract::BackgroundRemoverPort;
use crate::taxonomy::removal_types_vo::ModelType;

/// ONNX model input dimensions (RMBG-2.0 requires 1024x1024).
const ML_MODEL_IMAGE_WIDTH: u32 = 1024;
const ML_MODEL_IMAGE_HEIGHT: u32 = 1024;

/// ImageNet normalization mean (RMBG-2.0 expects ImageNet stats).
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Edge sharpening thresholds for alpha mask refinement.
const EDGE_LOW_THRESHOLD: f32 = 0.15;
const EDGE_HIGH_THRESHOLD: f32 = 0.85;

struct CachedSession {
    model_path: String,
    session: Arc<Mutex<ort::session::Session>>,
}

pub struct OnnxRemoverAdapter {
    enabled: bool,
    cache: Mutex<Option<CachedSession>>,
}

impl OnnxRemoverAdapter {
    pub fn new() -> Self {
        Self {
            enabled: true,
            cache: Mutex::new(None),
        }
    }

    fn load_or_reuse_session(&self, model_path: &Path) -> anyhow::Result<Arc<Mutex<ort::session::Session>>> {
        let model_path_str = model_path.to_string_lossy().to_string();

        let mut cache = self.cache.lock()
            .expect("Failed to lock model cache mutex");
        if let Some(ref cached) = *cache && cached.model_path == model_path_str {
            return Ok(Arc::clone(&cached.session));
        }
        let mut builder = ort::session::Session::builder()?;

        #[cfg(feature = "rocm")]
        {
            use ort::ep::rocm::ROCm;
            builder = builder
                .with_execution_providers([ROCm::default().build()])
                .map_err(|e| anyhow!("ROCm EP error: {:?}", e))?;
        }
        let session = Arc::new(Mutex::new(builder
            .commit_from_file(model_path)
            .map_err(|e| anyhow!("Failed to load model: {:?}", e))?));

        *cache = Some(CachedSession {
            model_path: model_path_str,
            session: Arc::clone(&session),
        });

        Ok(session)
    }

    fn resize_rgba(img: &DynamicImage, target_width: u32, target_height: u32) -> anyhow::Result<Vec<u8>> {
        let width = NonZeroU32::new(img.width()).ok_or(anyhow!("Invalid image width"))?;
        let height = NonZeroU32::new(img.height()).ok_or(anyhow!("Invalid image height"))?;
        let mut src_image = fr::Image::from_vec_u8(
            width, height,
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

    fn preprocess_image(image: &DynamicImage) -> anyhow::Result<Array3<f32>> {
        let img_vec = Self::resize_rgba(image, ML_MODEL_IMAGE_WIDTH, ML_MODEL_IMAGE_HEIGHT)?;
        let size = (ML_MODEL_IMAGE_WIDTH * ML_MODEL_IMAGE_HEIGHT) as usize;
        let mut r_vec = Vec::with_capacity(size);
        let mut g_vec = Vec::with_capacity(size);
        let mut b_vec = Vec::with_capacity(size);

        for chunk in img_vec.chunks(4) {
            r_vec.push(chunk[0]);
            g_vec.push(chunk[1]);
            b_vec.push(chunk[2]);
        }

        let reordered_vec = [r_vec, g_vec, b_vec].concat();
        let img_ndarray = Array3::from_shape_vec(
            (3, ML_MODEL_IMAGE_WIDTH as usize, ML_MODEL_IMAGE_HEIGHT as usize),
            reordered_vec,
        )?;

        let img_float: Array3<f32> = img_ndarray.mapv(|x| x as f32 / 255.0);
        Ok(Self::normalize_image(&img_float))
    }

    fn normalize_image(img: &Array3<f32>) -> Array3<f32> {
        let mut normalized = img.clone();
        for c in 0..3 {
            let mut channel_view = normalized.slice_mut(s![c, .., ..]);
            channel_view.mapv_inplace(|x| (x - IMAGENET_MEAN[c]) / IMAGENET_STD[c]);
        }
        normalized
    }

    fn postprocess_image(model_result: &ndarray::ArrayView2<f32>) -> anyhow::Result<DynamicImage> {
        let sigmoid_result = model_result.mapv(|x| 1.0 / (1.0 + (-x).exp()));

        let ma = sigmoid_result.iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or(anyhow!("Failed to find max value in output tensor"))?;
        let mi = sigmoid_result.iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or(anyhow!("Failed to find min value in output tensor"))?;
        let result = (sigmoid_result.mapv(|x| x - mi) / (ma - mi)) * 255.0;

        let result_u8 = result.mapv(|x| x as u8).into_raw_vec_and_offset().0;
        let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(ML_MODEL_IMAGE_WIDTH, ML_MODEL_IMAGE_HEIGHT);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let index = (y * ML_MODEL_IMAGE_WIDTH + x) as usize;
            *pixel = Rgb([result_u8[index], result_u8[index], result_u8[index]]);
        }

        Ok(DynamicImage::ImageRgb8(imgbuf))
    }

    fn apply_mask(original_image: &DynamicImage, mask_image: &DynamicImage) -> DynamicImage {
        let mut no_bg_image: RgbaImage = ImageBuffer::new(mask_image.width(), mask_image.height());

        for (x, y, pixel) in no_bg_image.enumerate_pixels_mut() {
            let orig_pixel = original_image.get_pixel(x, y);
            let raw_alpha = mask_image.get_pixel(x, y)[0] as f32 / 255.0;

            let sharpened_alpha = if raw_alpha < EDGE_LOW_THRESHOLD {
                0.0
            } else if raw_alpha > EDGE_HIGH_THRESHOLD {
                1.0
            } else {
                (raw_alpha - EDGE_LOW_THRESHOLD) / (EDGE_HIGH_THRESHOLD - EDGE_LOW_THRESHOLD)
            };

            let smooth_alpha = sharpened_alpha * sharpened_alpha * (3.0 - 2.0 * sharpened_alpha);
            let alpha_u8 = (smooth_alpha * 255.0).round() as u8;

            if alpha_u8 == 0 {
                *pixel = Rgba([0, 0, 0, 0]);
            } else {
                let r = if raw_alpha > 0.01 {
                    ((orig_pixel[0] as f32 / raw_alpha).round() as u32).min(255) as u8
                } else {
                    orig_pixel[0]
                };
                let g = if raw_alpha > 0.01 {
                    ((orig_pixel[1] as f32 / raw_alpha).round() as u32).min(255) as u8
                } else {
                    orig_pixel[1]
                };
                let b = if raw_alpha > 0.01 {
                    ((orig_pixel[2] as f32 / raw_alpha).round() as u32).min(255) as u8
                } else {
                    orig_pixel[2]
                };

                *pixel = Rgba([r, g, b, alpha_u8]);
            }
        }
        DynamicImage::ImageRgba8(no_bg_image)
    }
}

impl Default for OnnxRemoverAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundRemoverPort for OnnxRemoverAdapter {
    fn remove_background(
        &self,
        model_path: &Path,
        input_path: &Path,
        output_path: &Path,
        _model_type: &ModelType,
    ) -> anyhow::Result<()> {
        if !self.enabled {
            anyhow::bail!("ONNX remover adapter is disabled");
        }
        use indicatif::ProgressBar;
        use std::time::Instant;

        let pb = ProgressBar::new_spinner();
        pb.set_message("Loading AI model into ONNX Runtime...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));

        let start_time = Instant::now();

        let model_arc = self.load_or_reuse_session(model_path)?;
        let mut model = model_arc.lock()
            .map_err(|e| anyhow!("Failed to lock model mutex: {:?}", e))?;

        let input_name = model.inputs()[0].name().to_string();
        let output_name = model.outputs().last()
            .ok_or(anyhow!("Model has no output nodes"))?
            .name().to_string();

        pb.set_message("Opening and reading input image...");

        let original_img = image::open(input_path)
            .context("Failed to open input image")?;

        pb.set_message("Removing background (AI Inference)...");

        let img = Self::preprocess_image(&original_img)?;
        let input = img.insert_axis(Axis(0));
        let input_tensor = ort::value::Tensor::from_array(input.clone())
            .map_err(|e| anyhow!("Failed to create input tensor: {:?}", e))?;
        let inputs = ort::inputs![&input_name => input_tensor];

        let outputs = model.run(inputs)
            .map_err(|e| anyhow!("Failed to run model: {:?}", e))?;

        let (shape, slice) = outputs[output_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|e| anyhow!("Failed to extract output tensor: {:?}", e))?;

        let output_view = ndarray::ArrayView4::from_shape(
            (shape[0] as usize, shape[1] as usize, shape[2] as usize, shape[3] as usize),
            slice,
        ).map_err(|e| anyhow!("Invalid output tensor shape: {}", e))?;

        let output_2d = output_view.slice(s![0, 0, .., ..]);
        let image = Self::postprocess_image(&output_2d)?;

        let (original_width, original_height) = (original_img.width(), original_img.height());
        let resized = Self::resize_rgba(&image, original_width, original_height)?;
        let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            original_width, original_height, resized,
        ).ok_or(anyhow!("Failed to create image buffer for resized mask"))?;
        let mask = DynamicImage::ImageRgba8(img_buffer);

        let img_without_bg = Self::apply_mask(&original_img, &mask);

        pb.set_message("Saving resulting image...");
        img_without_bg.save(output_path)
            .context("Failed to save output image")?;

        pb.finish_and_clear();

        let duration = start_time.elapsed();
        println!(
            "{} Successfully removed background in {:.2?}!",
            "SUCCESS:".green().bold(),
            duration
        );
        println!(
            "{} Result saved to: {}",
            "OUTPUT:".green().bold(),
            output_path.to_string_lossy().underline()
        );

        Ok(())
    }
}

#[cfg(test)]
#[path = "../../test/onnx_remover_tests.rs"]
mod tests;
