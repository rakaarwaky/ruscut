use std::num::NonZeroU32;
use std::sync::Arc;
use anyhow::{anyhow, Context};
use fast_image_resize as fr;
use image::{DynamicImage, ImageBuffer};
use crate::contract::{ModelDownloaderPort, OnnxRemoverPort, DirectAmdgpuRemoverPort, VideoProcessorPort, RemovalUseCaseProtocol};
use crate::taxonomy::removal_types_vo::{is_video_path, RemovalOptions};
use crate::taxonomy::{BenchmarkReportVo, EngineNameVo, ModelPathVo, TensorDataVo};

/// ONNX/Compute model input dimensions (BRIA RMBG-2.0 requires 1024x1024).
const ML_MODEL_IMAGE_WIDTH: u32 = 1024;
const ML_MODEL_IMAGE_HEIGHT: u32 = 1024;

/// ImageNet normalization mean & std (RMBG-2.0 expects ImageNet stats).
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Edge sharpening thresholds for alpha mask refinement.
const EDGE_LOW_THRESHOLD: f32 = 0.15;
const EDGE_HIGH_THRESHOLD: f32 = 0.85;

/// Core business use case that orchestrates model download and background removal.
pub struct RemovalUseCase {
    downloader: Arc<dyn ModelDownloaderPort>,
    onnx_remover: Arc<dyn OnnxRemoverPort>,
    direct_remover: Arc<dyn DirectAmdgpuRemoverPort>,
    video_processor: Arc<dyn VideoProcessorPort>,
}

impl RemovalUseCase {
    pub fn new(
        downloader: Arc<dyn ModelDownloaderPort>,
        onnx_remover: Arc<dyn OnnxRemoverPort>,
        direct_remover: Arc<dyn DirectAmdgpuRemoverPort>,
        video_processor: Arc<dyn VideoProcessorPort>,
    ) -> Self {
        Self { downloader, onnx_remover, direct_remover, video_processor }
    }

    /// Preprocesses image by resizing to 1024x1024 and creating normalized matrix buffer.
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

    /// Business Logic: Preprocesses DynamicImage into a flattened float array tensor.
    fn preprocess_image(image: &DynamicImage) -> anyhow::Result<Vec<f32>> {
        let img_vec = Self::resize_rgba(image, ML_MODEL_IMAGE_WIDTH, ML_MODEL_IMAGE_HEIGHT)?;
        let size = (ML_MODEL_IMAGE_WIDTH * ML_MODEL_IMAGE_HEIGHT) as usize;

        let mut tensor = vec![0.0f32; size * 3];

        for i in 0..size {
            let idx = i * 4;
            tensor[i] = (img_vec[idx] as f32 / 255.0 - IMAGENET_MEAN[0]) / IMAGENET_STD[0];
            tensor[size + i] = (img_vec[idx + 1] as f32 / 255.0 - IMAGENET_MEAN[1]) / IMAGENET_STD[1];
            tensor[2 * size + i] = (img_vec[idx + 2] as f32 / 255.0 - IMAGENET_MEAN[2]) / IMAGENET_STD[2];
        }

        Ok(tensor)
    }

    /// Business Logic: Postprocesses raw inference output into a grayscale mask DynamicImage.
    fn postprocess_image(model_result: &[f32]) -> anyhow::Result<DynamicImage> {
        let size = (ML_MODEL_IMAGE_WIDTH * ML_MODEL_IMAGE_HEIGHT) as usize;

        let mut sigmoid = Vec::with_capacity(size);
        let mut mi = f32::INFINITY;
        let mut ma = f32::NEG_INFINITY;

        for &v in model_result.iter().take(size) {
            let s = 1.0 / (1.0 + (-v).exp());
            if s > ma { ma = s; }
            if s < mi { mi = s; }
            sigmoid.push(s);
        }

        let range = if ma > mi { ma - mi } else { 1.0 };
        let mut raw = Vec::with_capacity(size * 3);
        for &s in sigmoid.iter() {
            let val = ((s - mi) / range * 255.0) as u8;
            raw.push(val);
            raw.push(val);
            raw.push(val);
        }

        let img = ImageBuffer::from_raw(ML_MODEL_IMAGE_WIDTH, ML_MODEL_IMAGE_HEIGHT, raw)
            .ok_or(anyhow!("Failed to create postprocess result image"))?;
        Ok(DynamicImage::ImageRgb8(img))
    }

    /// Business Logic: Refines alpha mask using edge-sharpening and blends with original image.
    fn apply_mask(original_image: &DynamicImage, mask_image: &DynamicImage) -> DynamicImage {
        let width = mask_image.width();
        let height = mask_image.height();
        let size = (width * height) as usize;

        let orig = original_image.to_rgba8().into_raw();
        let mask = mask_image.to_rgb8().into_raw();

        let mut out = Vec::with_capacity(size * 4);
        let inv_255 = 1.0 / 255.0;

        for i in 0..size {
            let r = orig[i * 4];
            let g = orig[i * 4 + 1];
            let b = orig[i * 4 + 2];
            let raw_alpha_val = mask[i * 3] as f32 * inv_255;

            let sharpened = if raw_alpha_val < EDGE_LOW_THRESHOLD {
                0.0
            } else if raw_alpha_val > EDGE_HIGH_THRESHOLD {
                1.0
            } else {
                (raw_alpha_val - EDGE_LOW_THRESHOLD) / (EDGE_HIGH_THRESHOLD - EDGE_LOW_THRESHOLD)
            };

            let smooth = sharpened * sharpened * (3.0 - 2.0 * sharpened);
            let alpha = (smooth * 255.0).round() as u8;

            if alpha == 0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
            } else if raw_alpha_val > 0.01 {
                let scale = 1.0 / raw_alpha_val;
                out.push(((r as f32 * scale).round() as u32).min(255) as u8);
                out.push(((g as f32 * scale).round() as u32).min(255) as u8);
                out.push(((b as f32 * scale).round() as u32).min(255) as u8);
                out.push(alpha);
            } else {
                out.push(r);
                out.push(g);
                out.push(b);
                out.push(alpha);
            }
        }

        let img = ImageBuffer::from_raw(width, height, out)
            .expect("Failed to create mask output image");
        DynamicImage::ImageRgba8(img)
    }

    /// Business Logic: Process a single frame/image background removal.
    fn process_single_frame(
        &self,
        model_path: &std::path::Path,
        input_path: &std::path::Path,
        output_path: &std::path::Path,
        _model_type: &crate::taxonomy::removal_types_vo::ModelType,
    ) -> anyhow::Result<()> {
        // 1. Open original image
        let original_img = image::open(input_path)
            .context("Failed to open input image")?;

        // 2. Preprocess to normalized flat float tensor (1024x1024x3)
        let img_tensor = Self::preprocess_image(&original_img)?;

        // 3. Dispatch raw inference to hardware tool (Infrastructure!)
        let model_vo = ModelPathVo::new(model_path.to_path_buf());
        let tensor_vo = TensorDataVo::new(img_tensor);
        let raw_output = if std::env::var("RUSCUT_DIRECT_GPU").is_ok() || std::env::var("RUSCUT_VULKAN").is_ok() {
            self.direct_remover.amdgpu_run_inference(&model_vo, &tensor_vo)?
        } else {
            self.onnx_remover.onnx_run_inference(&model_vo, &tensor_vo)?
        };

        // 4. Postprocess tensor back into transparent mask
        let mask_img = Self::postprocess_image(&raw_output.data)?;
        let final_img = Self::apply_mask(&original_img, &mask_img);

        // 5. Save resulting image to output path
        final_img.save(output_path)
            .context("Failed to save output image")?;

        Ok(())
    }
}

impl RemovalUseCaseProtocol for RemovalUseCase {
    fn usecase_execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        let model_path = if let Some(ref custom_path) = options.custom_model_path {
            if !custom_path.exists() {
                anyhow::bail!("Custom model file not found at path: {:?}", custom_path);
            }
            custom_path.clone()
        } else {
            self.downloader.downloader_ensure_model(&options.model_type, options.force_download)?
        };

        if is_video_path(&options.input_path) {
            self.video_processor.video_process_video(
                &options.input_path,
                &options.output_path,
                &|frame_in, frame_out| {
                    self.process_single_frame(&model_path, frame_in, frame_out, &options.model_type)
                }
            )?;
        } else {
            self.process_single_frame(&model_path, &options.input_path, &options.output_path, &options.model_type)?;
        }

        Ok(())
    }

    fn usecase_run_benchmark(&self) -> anyhow::Result<BenchmarkReportVo> {
        let model_type = crate::taxonomy::removal_types_vo::ModelType::Full;
        let model_path = self.downloader.downloader_ensure_model(&model_type, false)?;

        // Find or create input image
        let input_path = std::path::PathBuf::from("test/test_image.png");
        let final_input_path = if input_path.exists() {
            input_path
        } else {
            let mock_path = std::env::temp_dir().join("ruscut_bench_mock.png");
            if !mock_path.exists() {
                let img: image::RgbImage = image::ImageBuffer::from_pixel(1024, 1024, image::Rgb([255, 255, 255]));
                img.save(&mock_path)?;
            }
            mock_path
        };

        use std::time::Instant;

        let active_engine_name = self.usecase_get_engine_name();

        // Warm up by ensuring model is cached
        let _ = self.downloader.downloader_ensure_model(&model_type, false)?;

        let original_img = image::open(&final_input_path).context("Failed to open image")?;
        let (original_width, original_height) = (original_img.width(), original_img.height());

        // Warmup inference pass
        {
            let img_tensor = Self::preprocess_image(&original_img)?;
            let model_vo = ModelPathVo::new(model_path.clone());
            let tensor_vo = TensorDataVo::new(img_tensor);
            let raw_output = if std::env::var("RUSCUT_DIRECT_GPU").is_ok() || std::env::var("RUSCUT_VULKAN").is_ok() {
                self.direct_remover.amdgpu_run_inference(&model_vo, &tensor_vo)?
            } else {
                self.onnx_remover.onnx_run_inference(&model_vo, &tensor_vo)?
            };
            let _ = Self::postprocess_image(&raw_output.data)?;
        }

        let iterations = 10;
        let mut total_preprocess = std::time::Duration::default();
        let mut total_inference = std::time::Duration::default();
        let mut total_postprocess = std::time::Duration::default();
        let mut total_mask = std::time::Duration::default();
        let mut total_loop = std::time::Duration::default();

        for _ in 0..iterations {
            let loop_start = Instant::now();

            // 1. Preprocess
            let t_start = Instant::now();
            let img_tensor = Self::preprocess_image(&original_img)?;
            total_preprocess += t_start.elapsed();

            // 2. Inference
            let t_start = Instant::now();
            let model_vo = ModelPathVo::new(model_path.clone());
            let tensor_vo = TensorDataVo::new(img_tensor);
            let raw_output = if std::env::var("RUSCUT_DIRECT_GPU").is_ok() || std::env::var("RUSCUT_VULKAN").is_ok() {
                self.direct_remover.amdgpu_run_inference(&model_vo, &tensor_vo)?
            } else {
                self.onnx_remover.onnx_run_inference(&model_vo, &tensor_vo)?
            };
            total_inference += t_start.elapsed();

            // 3. Postprocess
            let t_start = Instant::now();
            let mask_img = Self::postprocess_image(&raw_output.data)?;
            total_postprocess += t_start.elapsed();

            // 4. Mask application
            let t_start = Instant::now();
            let resized = Self::resize_rgba(&mask_img, original_width, original_height)?;
            let img_buffer = ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                original_width, original_height, resized,
            ).ok_or(anyhow!("Failed to create mask buffer"))?;
            let mask = DynamicImage::ImageRgba8(img_buffer);
            let _final_img = Self::apply_mask(&original_img, &mask);
            total_mask += t_start.elapsed();

            let iter_dur = loop_start.elapsed();
            total_loop += iter_dur;
        }

        let avg_preprocess = total_preprocess / iterations;
        let avg_inference = total_inference / iterations;
        let avg_postprocess = total_postprocess / iterations;
        let avg_mask = total_mask / iterations;
        let avg_total = total_loop / iterations;
        let fps = 1.0 / avg_total.as_secs_f32();

        Ok(BenchmarkReportVo {
            preprocess_duration: avg_preprocess,
            inference_duration: avg_inference,
            postprocess_duration: avg_postprocess,
            mask_duration: avg_mask,
            total_duration: avg_total,
            fps,
            engine_name: active_engine_name,
        })
    }

    fn usecase_get_engine_name(&self) -> EngineNameVo {
        if std::env::var("RUSCUT_DIRECT_GPU").is_ok() || std::env::var("RUSCUT_VULKAN").is_ok() {
            self.direct_remover.amdgpu_get_engine_name()
        } else {
            self.onnx_remover.onnx_get_engine_name()
        }
    }
}
