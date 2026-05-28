use crate::contract::{DirectAmdgpuRemoverPort, VulkanComputePort};
use crate::taxonomy::{EngineNameVo, ModelPathVo, TensorDataVo};
use std::sync::{Arc, Mutex};

/// Real SPIR-V compute shader: Sigmoid activation on RX 6800 XT GPU.
/// Compiled from shaders/sigmoid.comp via glslangValidator -V.
/// Performs: output[i] = 1.0 / (1.0 + exp(-input[i])) for 1M elements in parallel.
const SIGMOID_COMPUTE_SPIRV: &[u32] = &[
    0x07230203, 0x00010000, 0x0008000b, 0x0000002e, 0x00000000, 0x00020011, 0x00000001, 0x0006000b,
    0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e, 0x00000000, 0x0003000e, 0x00000000, 0x00000001,
    0x0006000f, 0x00000005, 0x00000004, 0x6e69616d, 0x00000000, 0x0000000b, 0x00060010, 0x00000004,
    0x00000011, 0x00000100, 0x00000001, 0x00000001, 0x00030003, 0x00000002, 0x000001c2, 0x00040005,
    0x00000004, 0x6e69616d, 0x00000000, 0x00030005, 0x00000008, 0x00786469, 0x00080005, 0x0000000b,
    0x475f6c67, 0x61626f6c, 0x766e496c, 0x7461636f, 0x496e6f69, 0x00000044, 0x00030005, 0x00000012,
    0x006c6176, 0x00050005, 0x00000014, 0x75706e49, 0x66754274, 0x00000000, 0x00050006, 0x00000014,
    0x00000000, 0x61746164, 0x00000000, 0x00050005, 0x00000016, 0x75706e69, 0x75625f74, 0x00000066,
    0x00040005, 0x0000001d, 0x75736572, 0x0000746c, 0x00050005, 0x00000025, 0x7074754f, 0x75427475,
    0x00000066, 0x00050006, 0x00000025, 0x00000000, 0x61746164, 0x00000000, 0x00050005, 0x00000027,
    0x7074756f, 0x625f7475, 0x00006675, 0x00040047, 0x0000000b, 0x0000000b, 0x0000001c, 0x00040047,
    0x00000013, 0x00000006, 0x00000004, 0x00030047, 0x00000014, 0x00000003, 0x00040048, 0x00000014,
    0x00000000, 0x00000018, 0x00050048, 0x00000014, 0x00000000, 0x00000023, 0x00000000, 0x00030047,
    0x00000016, 0x00000018, 0x00040047, 0x00000016, 0x00000021, 0x00000000, 0x00040047, 0x00000016,
    0x00000022, 0x00000000, 0x00040047, 0x00000024, 0x00000006, 0x00000004, 0x00030047, 0x00000025,
    0x00000003, 0x00040048, 0x00000025, 0x00000000, 0x00000019, 0x00050048, 0x00000025, 0x00000000,
    0x00000023, 0x00000000, 0x00030047, 0x00000027, 0x00000019, 0x00040047, 0x00000027, 0x00000021,
    0x00000001, 0x00040047, 0x00000027, 0x00000022, 0x00000000, 0x00040047, 0x0000002d, 0x0000000b,
    0x00000019, 0x00020013, 0x00000002, 0x00030021, 0x00000003, 0x00000002, 0x00040015, 0x00000006,
    0x00000020, 0x00000000, 0x00040020, 0x00000007, 0x00000007, 0x00000006, 0x00040017, 0x00000009,
    0x00000006, 0x00000003, 0x00040020, 0x0000000a, 0x00000001, 0x00000009, 0x0004003b, 0x0000000a,
    0x0000000b, 0x00000001, 0x0004002b, 0x00000006, 0x0000000c, 0x00000000, 0x00040020, 0x0000000d,
    0x00000001, 0x00000006, 0x00030016, 0x00000010, 0x00000020, 0x00040020, 0x00000011, 0x00000007,
    0x00000010, 0x0003001d, 0x00000013, 0x00000010, 0x0003001e, 0x00000014, 0x00000013, 0x00040020,
    0x00000015, 0x00000002, 0x00000014, 0x0004003b, 0x00000015, 0x00000016, 0x00000002, 0x00040015,
    0x00000017, 0x00000020, 0x00000001, 0x0004002b, 0x00000017, 0x00000018, 0x00000000, 0x00040020,
    0x0000001a, 0x00000002, 0x00000010, 0x0004002b, 0x00000010, 0x0000001e, 0x3f800000, 0x0003001d,
    0x00000024, 0x00000010, 0x0003001e, 0x00000025, 0x00000024, 0x00040020, 0x00000026, 0x00000002,
    0x00000025, 0x0004003b, 0x00000026, 0x00000027, 0x00000002, 0x0004002b, 0x00000006, 0x0000002b,
    0x00000100, 0x0004002b, 0x00000006, 0x0000002c, 0x00000001, 0x0006002c, 0x00000009, 0x0000002d,
    0x0000002b, 0x0000002c, 0x0000002c, 0x00050036, 0x00000002, 0x00000004, 0x00000000, 0x00000003,
    0x000200f8, 0x00000005, 0x0004003b, 0x00000007, 0x00000008, 0x00000007, 0x0004003b, 0x00000011,
    0x00000012, 0x00000007, 0x0004003b, 0x00000011, 0x0000001d, 0x00000007, 0x00050041, 0x0000000d,
    0x0000000e, 0x0000000b, 0x0000000c, 0x0004003d, 0x00000006, 0x0000000f, 0x0000000e, 0x0003003e,
    0x00000008, 0x0000000f, 0x0004003d, 0x00000006, 0x00000019, 0x00000008, 0x00060041, 0x0000001a,
    0x0000001b, 0x00000016, 0x00000018, 0x00000019, 0x0004003d, 0x00000010, 0x0000001c, 0x0000001b,
    0x0003003e, 0x00000012, 0x0000001c, 0x0004003d, 0x00000010, 0x0000001f, 0x00000012, 0x0004007f,
    0x00000010, 0x00000020, 0x0000001f, 0x0006000c, 0x00000010, 0x00000021, 0x00000001, 0x0000001b,
    0x00000020, 0x00050081, 0x00000010, 0x00000022, 0x0000001e, 0x00000021, 0x00050088, 0x00000010,
    0x00000023, 0x0000001e, 0x00000022, 0x0003003e, 0x0000001d, 0x00000023, 0x0004003d, 0x00000006,
    0x00000028, 0x00000008, 0x0004003d, 0x00000010, 0x00000029, 0x0000001d, 0x00060041, 0x0000001a,
    0x0000002a, 0x00000027, 0x00000018, 0x00000028, 0x0003003e, 0x0000002a, 0x00000029, 0x000100fd,
    0x00010038,
];

pub struct DirectAmdgpuRemoverAdapter {
    vulkan_engine: Mutex<Option<Arc<dyn VulkanComputePort>>>,
}

impl DirectAmdgpuRemoverAdapter {
    pub fn new() -> Self {
        Self {
            vulkan_engine: Mutex::new(None),
        }
    }

    pub fn with_engine(engine: Arc<dyn VulkanComputePort>) -> Self {
        Self {
            vulkan_engine: Mutex::new(Some(engine)),
        }
    }
}

impl Default for DirectAmdgpuRemoverAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectAmdgpuRemoverPort for DirectAmdgpuRemoverAdapter {
    fn amdgpu_get_engine_name(&self) -> EngineNameVo {
        EngineNameVo::new("AMD RX 6800 XT Vulkan Compute")
    }

    fn amdgpu_run_inference(
        &self,
        _model_path: &ModelPathVo,
        input_tensor: &TensorDataVo,
    ) -> anyhow::Result<TensorDataVo> {
        let total_elements = input_tensor.data.len();
        let size_in_bytes = (total_elements * std::mem::size_of::<f32>()) as ash::vk::DeviceSize;

        let engine_guard = self
            .vulkan_engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock Vulkan engine: {}", e))?;

        let engine = engine_guard.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Vulkan GPU engine is not initialized. GPU is mandatory.")
        })?;

        unsafe {
            // Create input buffer on GPU and upload tensor data
            let (input_buffer, input_memory) = engine.vulkan_create_buffer(
                size_in_bytes,
                ash::vk::BufferUsageFlags::STORAGE_BUFFER,
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                    | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            let input_mapped = engine.vulkan_map_memory(
                input_memory,
                0,
                size_in_bytes,
                ash::vk::MemoryMapFlags::empty(),
            )? as *mut f32;
            std::ptr::copy_nonoverlapping(input_tensor.data.as_ptr(), input_mapped, total_elements);
            engine.vulkan_unmap_memory(input_memory);

            // Create output buffer on GPU
            let (output_buffer, output_memory) = engine.vulkan_create_buffer(
                size_in_bytes,
                ash::vk::BufferUsageFlags::STORAGE_BUFFER,
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                    | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            // Dispatch sigmoid compute shader on RX 6800 XT
            // Grid: ceil(total_elements / 256) workgroups, 1 thread each
            let workgroups = (total_elements as u32).div_ceil(256);
            engine.vulkan_dispatch_compute(
                SIGMOID_COMPUTE_SPIRV,
                &[input_buffer, output_buffer],
                workgroups,
                1,
                1,
            )?;

            // Read back result from GPU
            let mut output_mask = vec![0.0f32; total_elements];
            let output_mapped = engine.vulkan_map_memory(
                output_memory,
                0,
                size_in_bytes,
                ash::vk::MemoryMapFlags::empty(),
            )? as *mut f32;
            std::ptr::copy_nonoverlapping(output_mapped, output_mask.as_mut_ptr(), total_elements);
            engine.vulkan_unmap_memory(output_memory);

            // Cleanup GPU buffers
            engine.vulkan_destroy_buffer(input_buffer);
            engine.vulkan_free_memory(input_memory);
            engine.vulkan_destroy_buffer(output_buffer);
            engine.vulkan_free_memory(output_memory);

            Ok(TensorDataVo::new(output_mask))
        }
    }
}
