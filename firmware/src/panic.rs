//! Implements the panic handler

use core::{
    fmt::{self, Write},
    panic::PanicInfo,
};
use cortex_m::asm;

/// A static buffer to hold a formatted panic message
#[repr(C)]
struct PanicBuffer<const SIZE: usize> {
    /// The panic message
    pub message: [u8; SIZE],
    /// The position within the buffer
    pos: usize,
}
impl<const SIZE: usize> PanicBuffer<SIZE> {
    /// Creates a new empty panic buffer
    pub const fn new() -> Self {
        Self { message: [0; SIZE], pos: 0 }
    }
}
impl<const SIZE: usize> Write for PanicBuffer<SIZE> {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Get the target subbuffer
        let message = &mut self.message[self.pos..];
        let to_copy = core::cmp::min(s.len(), message.len());
        self.pos += to_copy;

        // Copy the string using a volatile write to ensure this is not optimized away
        let mut dest = message.as_mut_ptr();
        for source in s.bytes().take(to_copy) {
            unsafe { dest.write_volatile(source) };
            unsafe { dest = dest.add(1) };
        }
        Ok(())
    }
}
/// The static panic buffer
static mut PANIC_BUFFER: PanicBuffer<512> = PanicBuffer::new();

/// The panic handler
#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Write the message into the panic buffer and trigger a breakpoint
    let buffer = unsafe { &mut PANIC_BUFFER };
    let _ = write!(buffer, "{info}");
    asm::bkpt();

    // Raise a fatal exception
    asm::udf();
}
