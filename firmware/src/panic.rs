//! Implements the panic handler

use core::{
    fmt::{self, Display, Formatter, Write},
    hint,
    panic::PanicInfo,
    sync::atomic::{AtomicI8, Ordering::SeqCst},
};
use cortex_m::asm;
use cortex_m_rt::ExceptionFrame;
use rp_pico::hal::Sio;

/// A static buffer to hold a formatted panic message
#[repr(C)]
#[doc(hidden)]
pub struct PanicBuffer<const SIZE: usize> {
    /// The panic message
    message: [u8; SIZE],
    /// The size of the panic message
    len: usize,
}
impl<const SIZE: usize> PanicBuffer<SIZE> {
    /// Creates a new empty panic buffer
    pub const fn new() -> Self {
        Self { message: [0; SIZE], len: 0 }
    }
}
impl<const SIZE: usize> Display for PanicBuffer<SIZE> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Write the message
        for byte in self.message.iter().take(self.len) {
            // Escape the byte if necessary
            match byte.is_ascii_graphic() | byte.is_ascii_whitespace() {
                true => write!(f, "{}", *byte as char)?,
                false => write!(f, r#"\x{:02x}"#, byte)?,
            };
        }
        Ok(())
    }
}
impl<const SIZE: usize> Write for PanicBuffer<SIZE> {
    #[inline(never)]
    fn write_str(&mut self, str_: &str) -> fmt::Result {
        // Get the target subbuffer
        let message = &mut self.message[self.len..];
        let to_copy = core::cmp::min(str_.len(), message.len());
        self.len += to_copy;

        // Copy the string using a volatile write to ensure this is not optimized away
        let mut dest = message.as_mut_ptr();
        for source in str_.bytes().take(to_copy) {
            unsafe { dest.write_volatile(source) };
            unsafe { dest = dest.add(1) };
        }
        Ok(())
    }
}

/// The index of the core with the last panic (or `-1` in case there is no panic)
pub static LAST_PANIC: AtomicI8 = AtomicI8::new(-1);
/// The static panic buffers for each core
#[no_mangle]
#[doc(hidden)]
pub static mut PANIC_BUFFER: [PanicBuffer<512>; 2] = [PanicBuffer::new(), PanicBuffer::new()];

/// The panic handler
#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Set the panic flag
    let core = Sio::core();
    LAST_PANIC.store(core as i8, SeqCst);

    // Write the panic info into the buffer
    let buffer = unsafe { &mut PANIC_BUFFER[core as usize] };
    let _write_ok = write!(buffer, "{info}").is_ok();
    hint::black_box(buffer);

    // Trigger a breakpoint and raise a fatal exception
    asm::bkpt();
    asm::udf();
}

#[cortex_m_rt::exception]
#[allow(non_snake_case)]
unsafe fn DefaultHandler(irqn: i16) {
    loop {
        asm::bkpt();
        hint::black_box(irqn);
    }
}

#[cortex_m_rt::exception]
#[allow(non_snake_case)]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    loop {
        asm::bkpt();
        hint::black_box(ef);
    }
}
