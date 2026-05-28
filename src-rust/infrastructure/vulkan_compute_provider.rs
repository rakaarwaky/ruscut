use crate::contract::VulkanComputePort;
use crate::taxonomy::TensorDataVo;
use anyhow::anyhow;
use ash::{Device, Entry, Instance, vk};
use std::ffi::CString;

/// Custom Vulkan Compute Engine designed to run tensor operations directly on GPU.
pub struct VulkanComputeEngine {
    _entry: Entry,
    instance: Instance,
    pub device: Device,
    physical_device: vk::PhysicalDevice,
    compute_queue: vk::Queue,
    _queue_family_index: u32,
    command_pool: vk::CommandPool,
}

impl VulkanComputeEngine {
    /// Initializes Vulkan instance, detects GPU devices, and sets up a compute queue.
    /// Prioritizes AMD Radeon GPUs (Navi 21 / RX 6800 XT).
    pub fn new() -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load() }
            .map_err(|e| anyhow!("Failed to load Vulkan library: {:?}", e))?;

        let app_name = CString::new("Ruscut Compute Engine")?;
        let engine_name = CString::new("NoEngine")?;

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let instance_create_info = vk::InstanceCreateInfo::default().application_info(&app_info);

        let instance = unsafe { entry.create_instance(&instance_create_info, None) }
            .map_err(|e| anyhow!("Failed to create Vulkan instance: {:?}", e))?;

        // Enumerate physical devices and find the best GPU
        let physical_devices = unsafe { instance.enumerate_physical_devices() }
            .map_err(|e| anyhow!("Failed to find physical devices: {:?}", e))?;

        if physical_devices.is_empty() {
            anyhow::bail!("No Vulkan-compatible GPUs found!");
        }

        let mut selected_gpu = None;
        let mut selected_queue_family = None;
        let mut selected_device_type = vk::PhysicalDeviceType::default();

        for &phys_device in &physical_devices {
            let props = unsafe { instance.get_physical_device_properties(phys_device) };
            let device_name = unsafe {
                std::ffi::CStr::from_ptr(props.device_name.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            };
            let device_type = props.device_type;

            // Find a queue family that supports compute operations
            let queue_families =
                unsafe { instance.get_physical_device_queue_family_properties(phys_device) };
            let mut compute_family_idx = None;

            for (idx, family) in queue_families.iter().enumerate() {
                if family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                    compute_family_idx = Some(idx as u32);
                    break;
                }
            }

            if let Some(queue_idx) = compute_family_idx {
                let is_discrete = device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
                let is_amd = device_name.to_lowercase().contains("amd")
                    || device_name.to_lowercase().contains("radeon")
                    || props.vendor_id == 0x1002;

                // Priority: Discrete AMD GPU > Any Discrete GPU > Integrated AMD GPU > Any other
                let should_select = match (is_discrete, is_amd, selected_gpu.is_some()) {
                    (true, true, _) => true, // Always prefer discrete AMD
                    (true, false, true) => {
                        // Replace non-AMD discrete with any discrete? No
                        selected_device_type != vk::PhysicalDeviceType::DISCRETE_GPU
                    }
                    (true, false, false) => true, // First discrete GPU found
                    (false, true, false) => {
                        selected_device_type == vk::PhysicalDeviceType::CPU
                            || selected_device_type == vk::PhysicalDeviceType::OTHER
                            || selected_device_type == vk::PhysicalDeviceType::VIRTUAL_GPU
                    }
                    _ => false,
                };

                if should_select {
                    tracing::info!("Selecting GPU: {} (type: {:?})", device_name, device_type);
                    selected_gpu = Some((phys_device, device_name));
                    selected_queue_family = Some(queue_idx);
                    selected_device_type = device_type;
                }
            }
        }

        let (physical_device, _device_name) =
            selected_gpu.ok_or(anyhow!("No GPU found supporting compute operations!"))?;
        let queue_family_index =
            selected_queue_family.ok_or(anyhow!("No compute queue family found!"))?;

        // Create logical device
        let priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_create_info =
            vk::DeviceCreateInfo::default().queue_create_infos(std::slice::from_ref(&queue_info));

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }
            .map_err(|e| anyhow!("Failed to create Vulkan logical device: {:?}", e))?;

        let compute_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        // Create Command Pool for compute tasks
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe { device.create_command_pool(&pool_info, None) }
            .map_err(|e| anyhow!("Failed to create Vulkan command pool: {:?}", e))?;

        Ok(Self {
            _entry: entry,
            instance,
            device,
            physical_device,
            compute_queue,
            _queue_family_index: queue_family_index,
            command_pool,
        })
    }

    /// Allocates a GPU buffer sized to hold the given tensor data.
    ///
    /// # Safety
    /// Caller must ensure the returned buffer and memory are properly destroyed to prevent GPU memory leaks.
    pub unsafe fn create_tensor_buffer(
        &self,
        tensor: &TensorDataVo,
    ) -> anyhow::Result<(ash::vk::Buffer, ash::vk::DeviceMemory)> {
        let size = (tensor.len() * std::mem::size_of::<f32>()) as ash::vk::DeviceSize;
        unsafe {
            self.vulkan_create_buffer(
                size,
                ash::vk::BufferUsageFlags::STORAGE_BUFFER,
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                    | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
            )
        }
    }
}

impl VulkanComputePort for VulkanComputeEngine {
    /// Helper to allocate a GPU buffer (host-visible or device-local).
    ///
    /// # Safety
    /// Caller must ensure the Vulkan device is valid and memory properties are supported.
    unsafe fn vulkan_create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> anyhow::Result<(vk::Buffer, vk::DeviceMemory)> {
        unsafe {
            let buffer_info = vk::BufferCreateInfo::default()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let buffer = self
                .device
                .create_buffer(&buffer_info, None)
                .map_err(|e| anyhow!("Failed to create buffer: {:?}", e))?;

            let mem_requirements = self.device.get_buffer_memory_requirements(buffer);
            let mem_props = self
                .instance
                .get_physical_device_memory_properties(self.physical_device);

            let mut memory_type_index = None;
            for i in 0..mem_props.memory_type_count {
                if (mem_requirements.memory_type_bits & (1 << i)) != 0
                    && mem_props.memory_types[i as usize]
                        .property_flags
                        .contains(memory_properties)
                {
                    memory_type_index = Some(i);
                    break;
                }
            }

            let memory_type_index =
                memory_type_index.ok_or(anyhow!("Failed to find suitable Vulkan memory type!"))?;

            let alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_requirements.size)
                .memory_type_index(memory_type_index);

            let memory = self
                .device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| anyhow!("Failed to allocate device memory: {:?}", e))?;

            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|e| anyhow!("Failed to bind buffer memory: {:?}", e))?;

            Ok((buffer, memory))
        }
    }

    /// Submits a compute pipeline dispatch to run matrix operations or activations.
    unsafe fn vulkan_dispatch_compute(
        &self,
        shader_code: &[u32],
        buffers: &[vk::Buffer],
        grid_x: u32,
        grid_y: u32,
        grid_z: u32,
    ) -> anyhow::Result<()> {
        unsafe {
            // Create shader module
            let shader_info = vk::ShaderModuleCreateInfo::default().code(shader_code);
            let shader_module = self
                .device
                .create_shader_module(&shader_info, None)
                .map_err(|e| anyhow!("Failed to create compute shader module: {:?}", e))?;

            // Descriptor Set Layout definition
            let mut bindings = Vec::new();
            for (i, _) in buffers.iter().enumerate() {
                bindings.push(
                    vk::DescriptorSetLayoutBinding::default()
                        .binding(i as u32)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .descriptor_count(1)
                        .stage_flags(vk::ShaderStageFlags::COMPUTE),
                );
            }

            let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
            let descriptor_layout = self
                .device
                .create_descriptor_set_layout(&layout_info, None)
                .map_err(|e| anyhow!("Failed to create descriptor set layout: {:?}", e))?;

            // Pipeline Layout definition
            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(std::slice::from_ref(&descriptor_layout));
            let pipeline_layout = self
                .device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| anyhow!("Failed to create pipeline layout: {:?}", e))?;

            // Create Compute Pipeline
            let entry_point = CString::new("main")?;
            let stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::COMPUTE)
                .module(shader_module)
                .name(&entry_point);

            let pipeline_info = vk::ComputePipelineCreateInfo::default()
                .stage(stage_info)
                .layout(pipeline_layout);

            let pipelines = self
                .device
                .create_compute_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .map_err(|(_, e)| anyhow!("Failed to create compute pipeline: {:?}", e))?;
            let pipeline = pipelines[0];

            // Descriptor Pool and Descriptor Sets allocation
            let pool_size = vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(buffers.len() as u32);

            let pool_info = vk::DescriptorPoolCreateInfo::default()
                .max_sets(1)
                .pool_sizes(std::slice::from_ref(&pool_size));

            let descriptor_pool = self
                .device
                .create_descriptor_pool(&pool_info, None)
                .map_err(|e| anyhow!("Failed to create descriptor pool: {:?}", e))?;

            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(std::slice::from_ref(&descriptor_layout));

            let descriptor_sets = self
                .device
                .allocate_descriptor_sets(&alloc_info)
                .map_err(|e| anyhow!("Failed to allocate descriptor sets: {:?}", e))?;
            let descriptor_set = descriptor_sets[0];

            // Write descriptor sets mapping buffers
            let mut buffer_infos = Vec::new();
            let mut descriptor_writes = Vec::new();

            for &buffer in buffers {
                buffer_infos.push(
                    vk::DescriptorBufferInfo::default()
                        .buffer(buffer)
                        .offset(0)
                        .range(vk::WHOLE_SIZE),
                );
            }

            for (i, info) in buffer_infos.iter().enumerate() {
                descriptor_writes.push(
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(i as u32)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(std::slice::from_ref(info)),
                );
            }

            self.device.update_descriptor_sets(&descriptor_writes, &[]);

            // Record Command Buffer
            let cmd_alloc_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(self.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            let command_buffers = self
                .device
                .allocate_command_buffers(&cmd_alloc_info)
                .map_err(|e| anyhow!("Failed to allocate command buffers: {:?}", e))?;
            let cmd_buf = command_buffers[0];

            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device
                .begin_command_buffer(cmd_buf, &begin_info)
                .map_err(|e| anyhow!("Failed to begin recording command buffer: {:?}", e))?;

            self.device
                .cmd_bind_pipeline(cmd_buf, vk::PipelineBindPoint::COMPUTE, pipeline);
            self.device.cmd_bind_descriptor_sets(
                cmd_buf,
                vk::PipelineBindPoint::COMPUTE,
                pipeline_layout,
                0,
                std::slice::from_ref(&descriptor_set),
                &[],
            );

            self.device.cmd_dispatch(cmd_buf, grid_x, grid_y, grid_z);

            self.device
                .end_command_buffer(cmd_buf)
                .map_err(|e| anyhow!("Failed to record command buffer: {:?}", e))?;

            // Submit to Compute Queue and Wait
            let fence_info = vk::FenceCreateInfo::default();
            let fence = self
                .device
                .create_fence(&fence_info, None)
                .map_err(|e| anyhow!("Failed to create execution fence: {:?}", e))?;

            let submit_info =
                vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cmd_buf));

            self.device
                .queue_submit(
                    self.compute_queue,
                    std::slice::from_ref(&submit_info),
                    fence,
                )
                .map_err(|e| anyhow!("Failed to submit compute command: {:?}", e))?;

            self.device
                .wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX)
                .map_err(|e| anyhow!("Failed waiting for compute execution to finish: {:?}", e))?;

            // Cleanup temporary compute pipeline resources
            self.device.destroy_fence(fence, None);
            self.device.destroy_descriptor_pool(descriptor_pool, None);
            self.device.destroy_pipeline(pipeline, None);
            self.device.destroy_pipeline_layout(pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(descriptor_layout, None);
            self.device.destroy_shader_module(shader_module, None);
            self.device
                .free_command_buffers(self.command_pool, &[cmd_buf]);

            Ok(())
        }
    }

    unsafe fn vulkan_map_memory(
        &self,
        memory: ash::vk::DeviceMemory,
        offset: u64,
        size: u64,
        flags: ash::vk::MemoryMapFlags,
    ) -> anyhow::Result<*mut std::ffi::c_void> {
        unsafe {
            self.device
                .map_memory(memory, offset, size, flags)
                .map_err(|e| anyhow!("Failed to map device memory: {:?}", e))
        }
    }

    fn vulkan_unmap_memory(&self, memory: ash::vk::DeviceMemory) {
        unsafe {
            self.device.unmap_memory(memory);
        }
    }

    fn vulkan_destroy_buffer(&self, buffer: ash::vk::Buffer) {
        unsafe {
            self.device.destroy_buffer(buffer, None);
        }
    }

    fn vulkan_free_memory(&self, memory: ash::vk::DeviceMemory) {
        unsafe {
            self.device.free_memory(memory, None);
        }
    }
}

impl Drop for VulkanComputeEngine {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
