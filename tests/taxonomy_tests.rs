use ruscut::taxonomy::{EngineNameVo, ModelPathVo, TensorDataVo};
use std::path::PathBuf;

#[test]
fn test_engine_name_creation() {
    let name = EngineNameVo::new("ONNX Runtime");
    assert_eq!(name.as_str(), "ONNX Runtime");
}

#[test]
fn test_engine_name_display() {
    let name = EngineNameVo::new("Vulkan");
    assert_eq!(format!("{}", name), "Vulkan");
}

#[test]
fn test_engine_name_equality() {
    let a = EngineNameVo::new("FFmpeg");
    let b = EngineNameVo::new("FFmpeg");
    let c = EngineNameVo::new("ONNX");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_engine_name_clone() {
    let original = EngineNameVo::new("Custom Vulkan");
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_tensor_creation() {
    let tensor = TensorDataVo::new(vec![1.0, 2.0, 3.0]);
    assert_eq!(tensor.len(), 3);
    assert!(!tensor.is_empty());
}

#[test]
fn test_tensor_empty() {
    let tensor = TensorDataVo::new(vec![]);
    assert_eq!(tensor.len(), 0);
    assert!(tensor.is_empty());
}

#[test]
fn test_tensor_as_slice() {
    let tensor = TensorDataVo::new(vec![1.5, 2.5, 3.5]);
    let slice = tensor.as_slice();
    assert_eq!(slice, &[1.5, 2.5, 3.5]);
}

#[test]
fn test_tensor_clone() {
    let original = TensorDataVo::new(vec![1.0, 2.0]);
    let cloned = original.clone();
    assert_eq!(original.data, cloned.data);
}

#[test]
fn test_model_path_creation() {
    let path = ModelPathVo::new(PathBuf::from("/models/rmbg-2.0.onnx"));
    assert_eq!(
        path.as_path(),
        std::path::Path::new("/models/rmbg-2.0.onnx")
    );
}

#[test]
fn test_model_path_clone() {
    let original = ModelPathVo::new(PathBuf::from("/tmp/model.onnx"));
    let cloned = original.clone();
    assert_eq!(original.as_path(), cloned.as_path());
}
