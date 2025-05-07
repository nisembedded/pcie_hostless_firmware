#![no_std]
#![no_main]

use panic_halt as _;

use rp235x_hal as hal;

// Some things we need
use embedded_hal::delay::DelayNs;

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high frequency oscillator
const HFXO_FREQ: u32 = 12_000_000;

/// Entry point for the application
#[hal::entry]
fn main() -> ! {
    // Initialize the peripherals
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed for the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        HFXO_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls the GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set up the GPIO pins
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Waiting for a stabilize power processes
    timer.delay_ms(100);

    // Main loop
    loop {
        // Do something...
    }
}
