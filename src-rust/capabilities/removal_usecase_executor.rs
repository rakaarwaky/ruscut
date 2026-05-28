use crate::contract::ImageProcessorProtocol;
use crate::contract::RemovalUseCaseProtocol;
use crate::contract::{
    DirectAmdgpuRemoverPort, ModelDownloaderPort, OnnxRemoverPort, VideoProcessorPort,
};
use crate::taxonomy::removal_types_vo::{RemovalOptions, is_video_path};
use crate::taxonomy::{BenchmarkReportVo, EngineNameVo, ModelPathVo, TensorDataVo};
use anyhow::Context;
use image::{DynamicImage, ImageBuffer};
use std::sync::Arc;
use tracing::{Level, instrument};

/// Core business use case that orchestrates model download and background removal.
pub struct RemovalUseCase {
    downloader: Arc<dyn ModelDownloaderPort>,
    onnx_remover: Arc<dyn OnnxRemoverPort>,
    direct_remover: Arc<dyn DirectAmdgpuRemoverPort>,
    video_processor: Arc<dyn VideoProcessorPort>,
    image_processor: Arc<dyn ImageProcessorProtocol>,
    use_gpu: bool,
}

impl RemovalUseCase {
    pub fn new(
        downloader: Arc<dyn ModelDownloaderPort>,
        onnx_remover: Arc<dyn OnnxRemoverPort>,
        direct_remover: Arc<dyn DirectAmdgpuRemoverPort>,
        video_processor: Arc<dyn VideoProcessorPort>,
        image_processor: Arc<dyn ImageProcessorProtocol>,
    ) -> Self {
        let use_gpu =
            std::env::var("RUSCUT_DIRECT_GPU").is_ok() || std::env::var("RUSCUT_VULKAN").is_ok();
        Self {
            downloader,
            onnx_remover,
            direct_remover,
            video_processor,
            image_processor,
            use_gpu,
        }
    }

    pub fn new_with_gpu(
        downloader: Arc<dyn ModelDownloaderPort>,
        onnx_remover: Arc<dyn OnnxRemoverPort>,
        direct_remover: Arc<dyn DirectAmdgpuRemoverPort>,
        video_processor: Arc<dyn VideoProcessorPort>,
        image_processor: Arc<dyn ImageProcessorProtocol>,
        use_gpu: bool,
    ) -> Self {
        let use_gpu = use_gpu
            || std::env::var("RUSCUT_DIRECT_GPU").is_ok()
            || std::env::var("RUSCUT_VULKAN").is_ok();
        Self {
            downloader,
            onnx_remover,
            direct_remover,
            video_processor,
            image_processor,
            use_gpu,
        }
    }

    /// Process a single frame/image background removal.
    fn process_single_frame(
        &self,
        model_path: &std::path::Path,
        input_path: &std::path::Path,
        output_path: &std::path::Path,
        _model_type: &crate::taxonomy::removal_types_vo::ModelType,
    ) -> anyhow::Result<()> {
        let original_img = image::open(input_path).context("Failed to open input image")?;
        let img_tensor = self.image_processor.processor_preprocess(&original_img)?;

        let model_vo = ModelPathVo::new(model_path.to_path_buf());
        let tensor_vo = TensorDataVo::new(img_tensor);
        let raw_output = if self.use_gpu {
            self.direct_remover
                .amdgpu_run_inference(&model_vo, &tensor_vo)?
        } else {
            self.onnx_remover
                .onnx_run_inference(&model_vo, &tensor_vo)?
        };

        let mask_img = self
            .image_processor
            .processor_postprocess(&raw_output.data)?;
        let final_img = self
            .image_processor
            .processor_apply_mask(&original_img, &mask_img);

        final_img
            .save(output_path)
            .context("Failed to save output image")?;

        Ok(())
    }
}

impl RemovalUseCaseProtocol for RemovalUseCase {
    #[instrument(
        skip(self, options),
        fields(
            input = %options.input_path.display(),
            output = %options.output_path.display(),
            model = %options.model_type.label(),
            force = options.force_download
        ),
        err(level = Level::WARN)
    )]
    fn usecase_execute(&self, options: &RemovalOptions) -> anyhow::Result<()> {
        tracing::debug!("Starting background removal workflow");
        let start = std::time::Instant::now();
        let model_path = if let Some(ref custom_path) = options.custom_model_path {
            tracing::debug!(path = %custom_path.display(), "Using custom model path");
            if !custom_path.exists() {
                anyhow::bail!("Custom model file not found at path: {:?}", custom_path);
            }
            custom_path.clone()
        } else {
            self.downloader
                .downloader_ensure_model(&options.model_type, options.force_download)?
        };

        if is_video_path(&options.input_path) {
            tracing::info!("Processing video file");
            self.video_processor.video_process_video(
                &options.input_path,
                &options.output_path,
                &|frame_in, frame_out| {
                    self.process_single_frame(&model_path, frame_in, frame_out, &options.model_type)
                },
            )?;
        } else {
            tracing::info!("Processing image file");
            self.process_single_frame(
                &model_path,
                &options.input_path,
                &options.output_path,
                &options.model_type,
            )?;
        }

        let duration = start.elapsed();
        tracing::info!(
            duration_ms = duration.as_millis(),
            "Background removal completed"
        );

        Ok(())
    }

    fn usecase_run_benchmark(&self) -> anyhow::Result<BenchmarkReportVo> {
        use std::time::Instant;

        let model_type = crate::taxonomy::removal_types_vo::ModelType::Full;

        eprintln!("[benchmark] Ensuring model is cached...");
        let model_path = self
            .downloader
            .downloader_ensure_model(&model_type, false)?;
        eprintln!("[benchmark] Model ready: {}", model_path.display());

        // Use a real test image if available, otherwise generate a 1024x1024 mock
        let input_path = std::path::PathBuf::from("tests/fixtures/test_image.png");
        let final_input_path = if input_path.exists() {
            input_path
        } else {
            let mock_path = std::env::temp_dir().join("ruscut_bench_mock.png");
            if !mock_path.exists() {
                eprintln!("[benchmark] Generating 1024x1024 mock image...");
                let img: image::RgbImage =
                    image::ImageBuffer::from_pixel(1024, 1024, image::Rgb([128, 180, 220]));
                img.save(&mock_path)?;
            }
            mock_path
        };

        let active_engine_name = self.usecase_get_engine_name();
        eprintln!("[benchmark] Engine: {}", active_engine_name.as_str());

        let original_img =
            image::open(&final_input_path).context("Failed to open benchmark image")?;
        let (original_width, original_height) = (original_img.width(), original_img.height());
        eprintln!(
            "[benchmark] Image: {}x{} px",
            original_width, original_height
        );

        // Warmup pass — loads model into memory / warms up GPU driver
        eprintln!("[benchmark] Warming up (1 pass)...");
        {
            let img_tensor = self.image_processor.processor_preprocess(&original_img)?;
            let model_vo = ModelPathVo::new(model_path.clone());
            let tensor_vo = TensorDataVo::new(img_tensor);
            let raw_output = if self.use_gpu {
                self.direct_remover
                    .amdgpu_run_inference(&model_vo, &tensor_vo)?
            } else {
                self.onnx_remover
                    .onnx_run_inference(&model_vo, &tensor_vo)?
            };
            let _ = self
                .image_processor
                .processor_postprocess(&raw_output.data)?;
        }
        eprintln!("[benchmark] Warmup done. Running 10 timed iterations...");

        let iterations: u32 = 10;
        let mut total_preprocess = std::time::Duration::default();
        let mut total_inference = std::time::Duration::default();
        let mut total_postprocess = std::time::Duration::default();
        let mut total_mask = std::time::Duration::default();
        let mut total_loop = std::time::Duration::default();

        for i in 1..=iterations {
            eprint!("[benchmark] iteration {}/{}...", i, iterations);
            let loop_start = Instant::now();

            let t_start = Instant::now();
            let img_tensor = self.image_processor.processor_preprocess(&original_img)?;
            total_preprocess += t_start.elapsed();

            let t_start = Instant::now();
            let model_vo = ModelPathVo::new(model_path.clone());
            let tensor_vo = TensorDataVo::new(img_tensor);
            let raw_output = if self.use_gpu {
                self.direct_remover
                    .amdgpu_run_inference(&model_vo, &tensor_vo)?
            } else {
                self.onnx_remover
                    .onnx_run_inference(&model_vo, &tensor_vo)?
            };
            total_inference += t_start.elapsed();

            let t_start = Instant::now();
            let mask_img = self
                .image_processor
                .processor_postprocess(&raw_output.data)?;
            total_postprocess += t_start.elapsed();

            let t_start = Instant::now();
            let resized = self.image_processor.processor_resize(
                &mask_img,
                original_width,
                original_height,
            )?;
            let img_buffer = ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                original_width,
                original_height,
                resized,
            )
            .ok_or(anyhow::anyhow!("Failed to create mask buffer"))?;
            let mask = DynamicImage::ImageRgba8(img_buffer);
            let _final_img = self
                .image_processor
                .processor_apply_mask(&original_img, &mask);
            total_mask += t_start.elapsed();

            let iter_dur = loop_start.elapsed();
            total_loop += iter_dur;
            eprintln!(" done ({:.0?})", iter_dur);
        }
        eprintln!("[benchmark] All iterations complete.");

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
        if self.use_gpu {
            self.direct_remover.amdgpu_get_engine_name()
        } else {
            self.onnx_remover.onnx_get_engine_name()
        }
    }
}
