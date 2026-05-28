use std::path::Path;

/// Port contract for low-level PCI BAR Memory-Mapped I/O.
pub trait PciBarPort {
    /// Maps the PCI BAR region directly into process memory.
    ///
    /// # Safety
    /// This function is unsafe because it maps raw physical hardware addresses
    /// into the process space, which can lead to undefined behavior or segmentation
    /// faults if mapped incorrectly.
    unsafe fn pci_map_pci_bar(pci_path: &Path, size_bytes: usize) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Writes a 32-bit register value at the given offset.
    ///
    /// # Safety
    /// This function is unsafe because writing to arbitrary hardware memory offsets
    /// can cause unexpected state changes or device lockups.
    unsafe fn pci_write_reg(&self, offset_in_bytes: usize, value: u32);

    /// Reads a 32-bit register value at the given offset.
    ///
    /// # Safety
    /// This function is unsafe because reading from arbitrary hardware memory offsets
    /// can cause unexpected state transitions or segmentation faults.
    unsafe fn pci_read_reg(&self, offset_in_bytes: usize) -> u32;
    fn pci_is_simulated(&self) -> bool;
}
