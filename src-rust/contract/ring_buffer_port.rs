/// Port contract representing the circular Command Ring Buffer for AMD GPU command submission.
pub trait RingBufferPort {
    /// Write a single 32-bit dword into the ring buffer.
    fn ring_write(&mut self, value: u32);

    /// Write a sequence of 32-bit dwords (a full PM4 packet) into the ring buffer.
    fn ring_write_packet(&mut self, words: &[u32]);

    /// Ring the doorbell to notify the GPU Command Processor of new commands.
    fn ring_commit(&self);

    /// Returns the current write pointer offset within the ring buffer.
    fn ring_write_ptr(&self) -> usize;

    /// Returns true if this ring buffer operates in simulation mode (no real hardware).
    fn ring_is_simulated(&self) -> bool;

    /// Returns the total capacity of the ring buffer in dwords.
    fn ring_capacity(&self) -> usize;
}
