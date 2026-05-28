use std::time::Duration;
use crate::taxonomy::EngineNameVo;

/// Value Object representing the statistical report of a benchmark run.
/// Complies with strict three-word naming and mandatory _vo taxonomy suffix.
#[derive(Debug, Clone)]
pub struct BenchmarkReportVo {
    pub preprocess_duration: Duration,
    pub inference_duration: Duration,
    pub postprocess_duration: Duration,
    pub mask_duration: Duration,
    pub total_duration: Duration,
    pub fps: f32,
    pub engine_name: EngineNameVo,
}
