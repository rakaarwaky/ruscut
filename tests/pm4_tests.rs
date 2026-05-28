use ruscut::infrastructure::pm4_packet_loader::{Pm4Opcode, Pm4Packet};

#[test]
fn test_pm4_header_generation() {
    let packet = Pm4Packet::new(Pm4Opcode::Nop, vec![0xDEADBEEF]);
    let raw = packet.serialize();
    assert_eq!(raw.len(), 2);
    assert_eq!(raw[0] & (3 << 30), 3 << 30); // Must be Type 3
    assert_eq!((raw[0] >> 16) & 0x7F, Pm4Opcode::Nop.value() as u32); // Opcode check
    assert_eq!(raw[0] & 0x7FFF, 0); // Count check (1 dword - 1 = 0)
    assert_eq!(raw[1], 0xDEADBEEF);
}

#[test]
fn test_pm4_write_register() {
    let packet = Pm4Packet::write_register(0x1000, 0xABCD);
    let raw = packet.serialize();
    assert_eq!(raw.len(), 3); // header + reg_offset + value
    assert_eq!((raw[0] >> 16) & 0x7F, Pm4Opcode::WriteData.value() as u32);
    assert_eq!(raw[1], 0x1000);
    assert_eq!(raw[2], 0xABCD);
}

#[test]
fn test_pm4_nop() {
    let packet = Pm4Packet::new_nop(3);
    let raw = packet.serialize();
    assert_eq!(raw.len(), 4); // header + 3 nop dwords
    assert_eq!((raw[0] >> 16) & 0x7F, Pm4Opcode::Nop.value() as u32);
}

#[test]
fn test_pm4_serialize() {
    let packet = Pm4Packet::new(Pm4Opcode::DispatchDirect, vec![1, 2, 3]);
    let serialized = packet.serialize();
    assert_eq!(serialized.len(), 4);
}

#[test]
fn test_pm4_header_and_payload() {
    let packet = Pm4Packet::new(Pm4Opcode::Nop, vec![0xAA, 0xBB]);
    assert!(packet.header() != 0);
    assert_eq!(packet.payload(), &[0xAA, 0xBB]);
}
