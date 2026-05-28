/// Port contract representing the PM4 Packet Manifest 4 builder for AMD RDNA2 GPU.
pub trait Pm4PacketPort: Send + Sync {
    /// Serialize the packet into raw 32-bit words for GPU ring buffer submission.
    fn pm4_serialize(&self) -> Vec<u32>;

    /// Returns the raw 32-bit header word of this PM4 packet.
    fn pm4_header(&self) -> u32;

    /// Returns the payload (body) words of this PM4 packet, excluding the header.
    fn pm4_payload(&self) -> &[u32];

    /// Returns the total dword count (header + payload) of this packet.
    fn pm4_total_dwords(&self) -> usize {
        1 + self.pm4_payload().len()
    }

    /// Returns the PM4 opcode embedded in the header.
    fn pm4_opcode(&self) -> u8 {
        ((self.pm4_header() >> 16) & 0x7F) as u8
    }
}
