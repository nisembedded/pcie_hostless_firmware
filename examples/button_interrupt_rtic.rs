// Used for testing blinking with RTIC
#![no_std]
#![no_main]

use rtic_monotonics::rp235x::prelude::*;

use rp235x_hal as hal;

rp235x_timer_monotonic!(Mono);

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
const HFXO_FREQ: u32 = 12_000_000u32;

// Application
#[rtic::app(device = rp235x_hal::pac)]
mod app {
    use super::*;
    use embedded_hal_0_2::digital::v2::{InputPin, OutputPin, ToggleableOutputPin};
    use panic_halt as _;
    use rp235x_hal::{
        clocks,
        gpio::{
            self, bank0::Gpio25, bank0::Gpio45, FunctionSio, PullDown, PullNone, SioInput, SioOutput,
        },
        sio::Sio,
        watchdog::Watchdog,
    };

    #[shared]
    struct Shared {
        // Shared resources go here
        blinking: bool,
        button: gpio::Pin<Gpio45, FunctionSio<SioInput>, PullNone>,
    }
    #[local]
    struct Local {
        // Local resources go here
        led: gpio::Pin<Gpio25, FunctionSio<SioOutput>, PullDown>,
    }

    #[init()]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        // This is the entry point of the application

        // Set up the monotonic timer
        let _mono = Mono::start(ctx.device.TIMER0, &mut ctx.device.RESETS);

        // Set up the watchdog driver - needed for the clock setup code
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);

        // Configure the clocks
        let _clocks = clocks::init_clocks_and_plls(
            HFXO_FREQ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut ctx.device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        // Set up the GPIO pins
        let sio = Sio::new(ctx.device.SIO);
        let pins = gpio::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut ctx.device.RESETS,
        );
        let mut led = pins.gpio25.into_push_pull_output();
        led.set_low().unwrap();

        let button = pins.gpio45.reconfigure();
        button.set_interrupt_enabled(gpio::Interrupt::EdgeLow, true);

        unsafe {
            cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::IO_IRQ_BANK0);
        }

        // Spawn heartbeat task
        heartbeat::spawn().ok();

        let blinking = false;

        // Waiting for a stabilize power processes

        (Shared { button, blinking }, Local { led })
    }

    #[task(binds = IO_IRQ_BANK0, shared = [button, blinking])]
    fn gpio_irq(ctx: gpio_irq::Context) {
        // Handle GPIO interrupt
        let mut shared = ctx.shared.button;
        shared.lock(|button| {
            button.clear_interrupt(gpio::Interrupt::EdgeLow);
            if button.is_low().unwrap() {
                let mut blinking = ctx.shared.blinking;
                blinking.lock(|b| {
                    *b = !*b;
                });
            }
        });
    }

    #[task(local = [led], shared = [blinking])]
    async fn heartbeat(ctx: heartbeat::Context) {
        let mut blinking = ctx.shared.blinking;

        loop {
            blinking.lock(|b| {
                if *b {
                    ctx.local.led.toggle().unwrap();
                } else {
                    ctx.local.led.set_low().unwrap();
                }
            });

            Mono::delay(250.millis()).await;
            // Wait for 250ms
        }
    }
}
