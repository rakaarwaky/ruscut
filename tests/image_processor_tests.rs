use image::{DynamicImage, ImageBuffer, Rgb, Rgba};
use ruscut::capabilities::image_data_processor::ImageDataProcessor;
use ruscut::contract::ImageProcessorProtocol;

#[test]
fn test_preprocess_image_dimensions() {
    let img = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(256, 256, Rgba([255, 0, 0, 255])));

    let processor = ImageDataProcessor::new();
    let tensor = processor
        .processor_preprocess(&img)
        .expect("preprocess should succeed for valid 256x256 image");

    assert_eq!(tensor.len(), 1024 * 1024 * 3);
}

#[test]
fn test_postprocess_produces_valid_mask() {
    let mut raw_output = vec![0.0f32; 1024 * 1024];
    raw_output[0..100].copy_from_slice(&[5.0f32; 100]);

    let processor = ImageDataProcessor::new();
    let mask = processor
        .processor_postprocess(&raw_output)
        .expect("postprocess should succeed for valid tensor");

    assert_eq!(mask.width(), 1024);
    assert_eq!(mask.height(), 1024);
}

#[test]
fn test_preprocess_small_image() {
    let img = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(64, 64, Rgba([128, 128, 128, 255])));

    let processor = ImageDataProcessor::new();
    let tensor = processor
        .processor_preprocess(&img)
        .expect("preprocess should succeed for valid 64x64 image");
    assert_eq!(tensor.len(), 1024 * 1024 * 3);
}

#[test]
fn test_postprocess_edge_cases() {
    let processor = ImageDataProcessor::new();

    let zeros = vec![0.0f32; 1024 * 1024];
    let mask = processor
        .processor_postprocess(&zeros)
        .expect("postprocess should succeed for zero tensor");
    assert_eq!(mask.width(), 1024);

    let high = vec![100.0f32; 1024 * 1024];
    let mask = processor
        .processor_postprocess(&high)
        .expect("postprocess should succeed for high-value tensor");
    assert_eq!(mask.width(), 1024);
}

#[test]
fn test_apply_mask_basic() {
    let original = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
        100,
        100,
        Rgba([200, 150, 100, 255]),
    ));
    let mask = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(100, 100, Rgb([128, 128, 128])));

    let processor = ImageDataProcessor::new();
    let result = processor.processor_apply_mask(&original, &mask);
    assert_eq!(result.width(), 100);
    assert_eq!(result.height(), 100);
}
