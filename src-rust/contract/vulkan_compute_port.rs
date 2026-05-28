/// Port contract for Vulkan Compute operations.
pub trait VulkanComputePort: Send + Sync {
    /// Creates a raw Vulkan GPU memory buffer.
    ///
    /// # Safety
    /// This function is unsafe because it allocates memory directly from the GPU physical device
    /// which must be manually destroyed when no longer needed to prevent GPU memory leaks.
    unsafe fn vulkan_create_buffer(
        &self,
        size: ash::vk::DeviceSize,
        usage: ash::vk::BufferUsageFlags,
        memory_properties: ash::vk::MemoryPropertyFlags,
    ) -> anyhow::Result<(ash::vk::Buffer, ash::vk::DeviceMemory)>;

    /// Dispatches a compute shader grid using the provided shader code and buffer resources.
    ///
    /// # Safety
    /// This function is unsafe because dispatching custom hardware compute shaders can crash
    /// the GPU or cause device hangs/lost device states if the pipeline layout or buffer size is incorrect.
    unsafe fn vulkan_dispatch_compute(
        &self,
        shader_code: &[u32],
        buffers: &[ash::vk::Buffer],
        grid_x: u32,
        grid_y: u32,
        grid_z: u32,
    ) -> anyhow::Result<()>;

    /// Maps device memory into host address space.
    ///
    /// # Safety
    /// Caller must ensure the memory handle is valid and the size does not exceed allocation bounds.
    unsafe fn vulkan_map_memory(
        &self,
        memory: ash::vk::DeviceMemory,
        offset: u64,
        size: u64,
        flags: ash::vk::MemoryMapFlags,
    ) -> anyhow::Result<*mut std::ffi::c_void>;

    /// Unmaps a previously mapped device memory region.
    fn vulkan_unmap_memory(&self, memory: ash::vk::DeviceMemory);

    /// Destroys a Vulkan buffer and releases its associated resources.
    fn vulkan_destroy_buffer(&self, buffer: ash::vk::Buffer);

    /// Frees a previously allocated Vulkan device memory block.
    fn vulkan_free_memory(&self, memory: ash::vk::DeviceMemory);
}
