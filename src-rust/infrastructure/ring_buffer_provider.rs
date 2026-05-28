use crate::contract::RingBufferPort;
use crate::taxonomy::TensorDataVo;

/// Circular Command Ring Buffer for AMD GPU command submission (PM4 queues).
pub struct GpuRingBuffer {
    /// Pointer to mapped ring memory buffer.
    buffer: *mut u32,
    /// Size of the ring buffer in 32-bit dwords.
    size_dwords: usize,
    /// Current write pointer offset index.
    write_ptr: usize,
    /// Memory address of the Doorbell register for this queue.
    doorbell_address: *mut u32,
    /// Indicates whether in hardware simulation mode.
    simulated: bool,
}

impl GpuRingBuffer {
    /// Instantiates a new simulated GPU Ring Buffer for testing and fallback environments.
    pub fn new_simulated(size_dwords: usize) -> Self {
        Self {
            buffer: std::ptr::null_mut(),
            size_dwords,
            write_ptr: 0,
            doorbell_address: std::ptr::null_mut(),
            simulated: true,
        }
    }

    /// Creates a GPU Ring Buffer bound to actual mapped hardware VRAM and MMIO Doorbell addresses.
    ///
    /// # Safety
    /// Pointers must be valid mapped addresses from the physical PCIe MMIO/BAR space.
    pub unsafe fn new(
        buffer_ptr: *mut u32,
        size_dwords: usize,
        doorbell_ptr: *mut u32,
    ) -> Self {
        Self {
            buffer: buffer_ptr,
            size_dwords,
            write_ptr: 0,
            doorbell_address: doorbell_ptr,
            simulated: false,
        }
    }

    /// Submits a single 32-bit word command (PM4 header or payload word) into the ring.
    pub fn write(&mut self, value: u32) {
        if self.simulated {
            // Simulated ring buffer write
            self.write_ptr = (self.write_ptr + 1) % self.size_dwords;
        } else {
            unsafe {
                let offset = self.write_ptr % self.size_dwords;
                core::ptr::write_volatile(self.buffer.add(offset), value);
                self.write_ptr = (self.write_ptr + 1) % self.size_dwords;
            }
        }
    }

    /// Submits a series of PM4 packet commands into the ring.
    pub fn write_packet(&mut self, words: &[u32]) {
        for &word in words {
            self.write(word);
        }
    }

    /// Rings the Doorbell to notify the AMD GPU Command Processor (CP) of new compute jobs.
    pub fn commit(&self) {
        if self.simulated {
            // Simulated ring buffer commit
        } else {
            unsafe {
                // Write current write pointer to Doorbell register to wake GFX/Compute rings
                core::ptr::write_volatile(self.doorbell_address, self.write_ptr as u32);
            }
        }
    }

    pub fn write_ptr(&self) -> usize {
        self.write_ptr
    }

    pub fn is_simulated(&self) -> bool {
        self.simulated
    }

    pub fn capacity(&self) -> usize {
        self.size_dwords
    }

    /// Writes float tensor data as raw bit-pattern dwords into the ring buffer.
    pub fn write_tensor_data(&mut self, tensor: &TensorDataVo) {
        for &value in tensor.as_slice() {
            self.write(value.to_bits());
        }
    }
}

impl RingBufferPort for GpuRingBuffer {
    fn ring_write(&mut self, value: u32) {
        self.write(value);
    }

    fn ring_write_packet(&mut self, words: &[u32]) {
        self.write_packet(words);
    }

    fn ring_commit(&self) {
        self.commit();
    }

    fn ring_write_ptr(&self) -> usize {
        self.write_ptr()
    }

    fn ring_is_simulated(&self) -> bool {
        self.is_simulated()
    }

    fn ring_capacity(&self) -> usize {
        self.capacity()
    }
}
