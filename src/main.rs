#![no_std]
#![no_main]

use rtic_monotonics::rp235x::prelude::*;

use panic_halt as _;

use rp235x_hal as hal;

rp235x_timer_monotonic!(Mono);

// Some things we need
use embedded_hal::delay::DelayNs;

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[link_section = ".bi_entires"]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"PCIE Hostless"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];

/// External high frequency oscillator
const HFXO_FREQ: u32 = 12_000_000;

// Application
#[rtic::app(device = rp235x_hal::pac)]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        // Shared resources go here
    }
    #[local]
    struct Local {
        // Local resources go here
    }

    #[init()]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        // This is the entry point of the application

        // Set up the monotonic timer
        let _mono = Mono::start(ctx.device.TIMER0, &mut ctx.device.RESETS);

        // Initialize the peripherals
        let mut pac = hal::pac::Peripherals::take().unwrap();

        // Set up the watchdog driver - needed for the clock setup code
        let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

        // Configure the clocks
        let _clocks = hal::clocks::init_clocks_and_plls(
            HFXO_FREQ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .unwrap();

        // Set up the GPIO pins
        let sio = hal::Sio::new(pac.SIO);
        let _pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        // Waiting for a stabilize power processes

        (Shared {}, Local {})
    }
}
