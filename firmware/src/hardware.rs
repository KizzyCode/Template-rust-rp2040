//! Provides access handles for hardware/peripheral access

use cortex_m::delay::Delay;
use rp_pico::{
    hal::{
        clocks::{self, ClocksManager},
        gpio::{bank0::Gpio25, Pin, PinId},
        pac::{CorePeripherals, Peripherals},
        Clock, Sio, Watchdog,
    },
    Pins, XOSC_CRYSTAL_FREQ,
};

/// The GPIO pins
pub struct GpioPins {
    /// The LED pin
    pub led: Option<Pin<Gpio25, <Gpio25 as PinId>::Reset>>,
}

/// A hardware handle
pub struct Hardware {
    /// Watchdog peripheral
    pub watchdog: Watchdog,
    /// Abstraction layer providing clock management
    pub clocks: ClocksManager,
    /// System timer (SysTick) as a delay provider
    pub delay: Delay,
    /// BSP replacement for the HAL [`Pins`](rp2040_hal::gpio::Pins) type
    pub pins: GpioPins,
}
impl Hardware {
    /// Takes the hardware singleton
    pub fn take() -> Option<Self> {
        let peripherals = Peripherals::take()?;
        let core = CorePeripherals::take()?;
        Some(Self::new(peripherals, core))
    }

    /// Creates a new hardware handle
    pub fn new(peripherals: Peripherals, core: CorePeripherals) -> Self {
        // Destructure peripherals
        let Peripherals { CLOCKS, IO_BANK0, PADS_BANK0, PLL_SYS, PLL_USB, mut RESETS, SIO, WATCHDOG, XOSC, .. } =
            peripherals;
        let CorePeripherals { SYST, .. } = core;

        // Setup timing peripherals
        let mut watchdog = Watchdog::new(WATCHDOG);
        let clocks =
            clocks::init_clocks_and_plls(XOSC_CRYSTAL_FREQ, XOSC, CLOCKS, PLL_SYS, PLL_USB, &mut RESETS, &mut watchdog)
                .ok()
                .expect("failed to initialize clocks");
        let delay = Delay::new(SYST, clocks.system_clock.freq().to_Hz());

        // Setup GPIO peripherals
        let sio = Sio::new(SIO);
        let Pins { led, .. } = Pins::new(IO_BANK0, PADS_BANK0, sio.gpio_bank0, &mut RESETS);
        let pins = GpioPins { led: Some(led) };

        Self { clocks, watchdog, delay, pins }
    }
}
