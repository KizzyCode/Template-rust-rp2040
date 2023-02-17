#![no_std]
#![no_main]

mod hardware;
mod panic;

use embedded_hal::digital::v2::OutputPin;
use hardware::Hardware;

#[rp_pico::entry]
fn main() -> ! {
    // Grab our hardware objects
    let mut hardware = Hardware::take().expect("failed to initialize hardware");
    let led_pin = hardware.pins.led.take().expect("failed to access LED pin");

    // Blink the LED at 1 Hz
    let mut led_pin = led_pin.into_push_pull_output();
    loop {
        led_pin.set_high().expect("failed to set LED pin state");
        hardware.delay.delay_ms(500);
        led_pin.set_low().expect("failed to set LED pin state");
        hardware.delay.delay_ms(500);
    }
}
