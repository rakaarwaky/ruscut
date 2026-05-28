use crate::contract::PciBarPort;
use crate::taxonomy::ModelPathVo;
use anyhow::Context;
use core::ptr::{read_volatile, write_volatile};
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to disable unsafe operations in production/sandboxed environments.
static UNSAFE_OPERATIONS_ENABLED: AtomicBool = AtomicBool::new(true);

/// Call this at startup to disable unsafe hardware access.
/// Set RUSCUT_SAFE_MODE=1 environment variable to enable safe mode.
pub fn disable_unsafe_operations() {
    UNSAFE_OPERATIONS_ENABLED.store(false, Ordering::SeqCst);
    tracing::warn!("Unsafe hardware operations disabled - running in safe mode");
}

/// Checks if unsafe operations are allowed in the current environment.
pub fn is_unsafe_allowed() -> bool {
    UNSAFE_OPERATIONS_ENABLED.load(Ordering::SeqCst)
}

/// Vendor and Device ID for AMD Radeon RX 6800 XT (Navi 21 / gfx1030)
pub const AMD_VENDOR_ID: u16 = 0x1002;
pub const RX_6800XT_DEVICE_ID: u16 = 0x73bf;

/// Low-level Memory-Mapped I/O (MMIO) interface for GPU Register Space.
pub struct GpuRegisterSpace {
    /// Pointer to mapped physical address.
    base_address: *mut u32,
    /// Memory size of the mapped BAR.
    size: usize,
    /// Indicates whether we are in hardware simulation/mock mode.
    simulated: bool,
    /// Simulated register bank when hardware is not accessible.
    simulated_bank: std::cell::RefCell<Vec<u32>>,
}

impl GpuRegisterSpace {
    /// Runtime check before any unsafe operation.
    fn check_unsafe_allowed() -> anyhow::Result<()> {
        if !UNSAFE_OPERATIONS_ENABLED.load(Ordering::SeqCst) {
            anyhow::bail!(
                "Unsafe PCI operations are disabled in this environment (RUSCUT_SAFE_MODE)"
            );
        }
        if !cfg!(target_os = "linux") {
            anyhow::bail!("PCI BAR access only supported on Linux");
        }
        Ok(())
    }

    /// Creates a simulated register space for tests and environment fallbacks.
    pub fn new_simulated(size_bytes: usize) -> Self {
        let size_dwords = size_bytes / 4;
        Self {
            base_address: std::ptr::null_mut(),
            size: size_bytes,
            simulated: true,
            simulated_bank: std::cell::RefCell::new(vec![0; size_dwords]),
        }
    }

    /// Attempts to open and map the physical PCI register space of the GPU.
    ///
    /// # Safety
    /// Requires root privilege to read physical devices and map memory.
    /// This function performs runtime safety checks before proceeding.
    pub unsafe fn map_pci_bar(pci_path: &Path, size_bytes: usize) -> anyhow::Result<Self> {
        // Runtime safety check
        Self::check_unsafe_allowed()?;

        if !pci_path.exists() {
            tracing::debug!("PCI path not found, using simulated mode");
            return Ok(Self::new_simulated(size_bytes));
        }

        // Open PCI device resource (representing the MMIO BAR)
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(pci_path)
            .context("Failed to open PCI device. Requires root privileges.")?;

        let fd = std::os::unix::io::AsRawFd::as_raw_fd(&file);

        // Map the BAR into user memory using libc::mmap
        let map_addr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                size_bytes,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };

        if map_addr == libc::MAP_FAILED {
            let err = std::io::Error::last_os_error();
            tracing::error!(
                errno = err.raw_os_error(),
                "mmap failed for PCI BAR. This may require: (1) root privileges, (2) correct PCI path, (3) IOMMU disabled"
            );
            anyhow::bail!(
                "Failed to mmap PCI BAR: {} (errno: {})",
                err,
                err.raw_os_error().unwrap_or(-1)
            );
        }

        Ok(Self {
            base_address: map_addr as *mut u32,
            size: size_bytes,
            simulated: false,
            simulated_bank: std::cell::RefCell::new(Vec::new()),
        })
    }

    /// Write a 32-bit value to a specific offset (in bytes) in the register BAR.
    ///
    /// # Safety
    /// This function is unsafe because writing to arbitrary physical hardware memory offsets
    /// can cause unexpected state changes or hardware lockups.
    pub unsafe fn write_reg(&self, offset_in_bytes: usize, value: u32) {
        let dword_offset = offset_in_bytes / 4;
        if self.simulated {
            let mut bank = self.simulated_bank.borrow_mut();
            if dword_offset < bank.len() {
                bank[dword_offset] = value;
            }
        } else {
            unsafe {
                let reg_ptr = (self.base_address as *mut u8).add(offset_in_bytes) as *mut u32;
                write_volatile(reg_ptr, value);
            }
        }
    }

    /// Read a 32-bit value from a specific offset (in bytes) in the register BAR.
    ///
    /// # Safety
    /// This function is unsafe because reading from arbitrary physical hardware memory offsets
    /// can cause unexpected state transitions or segmentation faults.
    pub unsafe fn read_reg(&self, offset_in_bytes: usize) -> u32 {
        let dword_offset = offset_in_bytes / 4;
        if self.simulated {
            let bank = self.simulated_bank.borrow();
            if dword_offset < bank.len() {
                bank[dword_offset]
            } else {
                0
            }
        } else {
            unsafe {
                let reg_ptr = (self.base_address as *const u8).add(offset_in_bytes) as *const u32;
                read_volatile(reg_ptr)
            }
        }
    }

    pub fn is_simulated(&self) -> bool {
        self.simulated
    }

    /// Maps PCI BAR using a validated model path for device identification.
    ///
    /// # Safety
    /// Requires root privileges to access physical PCI device resources.
    pub unsafe fn from_model_path(path: &ModelPathVo, size_bytes: usize) -> anyhow::Result<Self> {
        unsafe { Self::map_pci_bar(path.as_path(), size_bytes) }
    }
}

impl Drop for GpuRegisterSpace {
    fn drop(&mut self) {
        if !self.simulated && !self.base_address.is_null() {
            unsafe {
                libc::munmap(self.base_address as *mut libc::c_void, self.size);
            }
        }
    }
}

impl PciBarPort for GpuRegisterSpace {
    unsafe fn pci_map_pci_bar(pci_path: &Path, size_bytes: usize) -> anyhow::Result<Self> {
        unsafe { Self::map_pci_bar(pci_path, size_bytes) }
    }

    unsafe fn pci_write_reg(&self, offset_in_bytes: usize, value: u32) {
        unsafe { self.write_reg(offset_in_bytes, value) }
    }

    unsafe fn pci_read_reg(&self, offset_in_bytes: usize) -> u32 {
        unsafe { self.read_reg(offset_in_bytes) }
    }

    fn pci_is_simulated(&self) -> bool {
        self.is_simulated()
    }
}
