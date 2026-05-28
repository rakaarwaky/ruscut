use ruscut::infrastructure::pci_bar_provider::{
    AMD_VENDOR_ID, GpuRegisterSpace, RX_6800XT_DEVICE_ID,
};
use ruscut::infrastructure::ring_buffer_provider::GpuRingBuffer;
use ruscut::taxonomy::TensorDataVo;

// ========== Ring Buffer Tests ==========

#[test]
fn test_simulated_ring_buffer_creation() {
    let ring = GpuRingBuffer::new_simulated(1024);
    assert!(ring.is_simulated());
    assert_eq!(ring.capacity(), 1024);
    assert_eq!(ring.write_ptr(), 0);
}

#[test]
fn test_simulated_write_advances_ptr() {
    let mut ring = GpuRingBuffer::new_simulated(16);
    ring.write(0xDEADBEEF);
    assert_eq!(ring.write_ptr(), 1);
    ring.write(0xCAFEBABE);
    assert_eq!(ring.write_ptr(), 2);
}

#[test]
fn test_simulated_write_wraps_around() {
    let mut ring = GpuRingBuffer::new_simulated(4);
    for i in 0..8u32 {
        ring.write(i);
    }
    assert_eq!(ring.write_ptr(), 0);
}

#[test]
fn test_write_packet() {
    let mut ring = GpuRingBuffer::new_simulated(16);
    let packet = vec![0x11111111, 0x22222222, 0x33333333];
    ring.write_packet(&packet);
    assert_eq!(ring.write_ptr(), 3);
}

#[test]
fn test_commit_simulated_no_panic() {
    let ring = GpuRingBuffer::new_simulated(16);
    ring.commit();
}

#[test]
fn test_write_tensor_data() {
    let mut ring = GpuRingBuffer::new_simulated(16);
    let tensor = TensorDataVo::new(vec![1.0f32, 2.0, 3.0]);
    ring.write_tensor_data(&tensor);
    assert_eq!(ring.write_ptr(), 3);
}

// ========== PCI BAR Tests ==========

#[test]
fn test_simulated_register_space() {
    let space = GpuRegisterSpace::new_simulated(4096);
    assert!(space.is_simulated());
}

#[test]
fn test_simulated_write_read() {
    let space = GpuRegisterSpace::new_simulated(4096);
    unsafe {
        space.write_reg(0, 0xDEADBEEF);
        assert_eq!(space.read_reg(0), 0xDEADBEEF);
    }
}

#[test]
fn test_simulated_multiple_registers() {
    let space = GpuRegisterSpace::new_simulated(4096);
    unsafe {
        space.write_reg(0, 0x11111111);
        space.write_reg(4, 0x22222222);
        space.write_reg(8, 0x33333333);
        assert_eq!(space.read_reg(0), 0x11111111);
        assert_eq!(space.read_reg(4), 0x22222222);
        assert_eq!(space.read_reg(8), 0x33333333);
    }
}

#[test]
fn test_simulated_out_of_bounds_read() {
    let space = GpuRegisterSpace::new_simulated(16);
    unsafe {
        assert_eq!(space.read_reg(100), 0);
    }
}

#[test]
fn test_vendor_device_ids() {
    assert_eq!(AMD_VENDOR_ID, 0x1002);
    assert_eq!(RX_6800XT_DEVICE_ID, 0x73BF);
}
