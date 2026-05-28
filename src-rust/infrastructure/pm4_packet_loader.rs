use crate::contract::Pm4PacketPort;
use crate::taxonomy::TensorDataVo;

/// Common PM4 Type 3 packet opcodes for AMD RDNA2 GPU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pm4Opcode {
    /// Write multiple registers
    WriteData = 0x37,
    /// Set a single or range of configuration registers
    SetConfigReg = 0x28,
    /// Dispatch compute grid (GFX10.3 / RDNA2 Compute Dispatch)
    DispatchDirect = 0x15,
    /// Direct Memory Access (DMA) Copy command
    DmaCopy = 0x50,
    /// Wait for register or memory state change (Sync)
    WaitRegMem = 0x3C,
    /// NOP (No operation)
    Nop = 0x10,
}

impl Pm4Opcode {
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

/// Type-safe PM4 Packet Manifest 4 builder for AMD RDNA2 GPU command queue submission.
#[derive(Debug, Clone)]
pub struct Pm4Packet {
    header: u32,
    payload: Vec<u32>,
}

impl Pm4Packet {
    /// Creates a new PM4 Type 3 packet with a specific opcode.
    pub fn new(opcode: Pm4Opcode, payload: Vec<u32>) -> Self {
        let count = if payload.is_empty() {
            0
        } else {
            (payload.len() - 1) as u16
        };

        // AMD PM4 Type 3 Header Format:
        // Bits [31:30] = 3 (Type 3 Packet)
        // Bits [22:16] = Opcode
        // Bits [14:0]  = Count (DWords in payload - 1)
        let header = (3 << 30) | ((opcode.value() as u32) << 16) | (count as u32);

        Self { header, payload }
    }

    /// Creates a NOP packet.
    pub fn new_nop(count: usize) -> Self {
        Self::new(Pm4Opcode::Nop, vec![0; count])
    }

    /// Creates a register write packet.
    pub fn write_register(reg_offset: u32, value: u32) -> Self {
        Self::new(Pm4Opcode::WriteData, vec![reg_offset, value])
    }

    /// Serializes the packet into 32-bit words for submission to the GpuRingBuffer.
    pub fn serialize(&self) -> Vec<u32> {
        let mut words = Vec::with_capacity(1 + self.payload.len());
        words.push(self.header);
        words.extend_from_slice(&self.payload);
        words
    }

    /// Gets the raw header value.
    pub fn header(&self) -> u32 {
        self.header
    }

    /// Gets the payload slice.
    pub fn payload(&self) -> &[u32] {
        &self.payload
    }

    /// Creates a WriteData packet that writes tensor float values as raw bits to a GPU register.
    pub fn write_tensor_data(reg_offset: u32, tensor: &TensorDataVo) -> Self {
        let mut words = vec![reg_offset];
        for &value in tensor.as_slice() {
            words.push(value.to_bits());
        }
        Self::new(Pm4Opcode::WriteData, words)
    }
}

impl Pm4PacketPort for Pm4Packet {
    fn pm4_serialize(&self) -> Vec<u32> {
        self.serialize()
    }

    fn pm4_header(&self) -> u32 {
        self.header()
    }

    fn pm4_payload(&self) -> &[u32] {
        self.payload()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
