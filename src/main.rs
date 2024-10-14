//! The starter code slowly blinks the LED and sets up
//! USB logging. It periodically logs messages over USB.
//!
//! This template uses [RTIC v2](https://rtic.rs/2/book/en/)
//! for structuring the application.

#![no_std]
#![no_main]

use teensy4_panic as _;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use bsp::board;
    use bsp::{
        hal::{gpio, iomuxc},
        pins,
    };
    use teensy4_bsp as bsp;

    use imxrt_log as logging;

    // Using Teensy 4.1
    use board::t41 as my_board;

    use rtic_monotonics::systick::{Systick, *};

    /// input on Pin 34
    type Input =  gpio::Input<pins::t41::P34>;

    /// pull up value should be twice input in impedance
    const PIN_CONFIG: iomuxc::Config = iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pullup47k));

    /// There are no resources shared across tasks.
    #[shared]
    struct Shared {}

    /// These resources are local to individual tasks.
    #[local]
    struct Local {
        /// The LED on pin 13.
        led: board::Led,
        /// A poller to control USB logging.
        poller: logging::Poller,
        /// The input pin
        input: Input,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut gpio2,
            mut pins,
            usb,
            ..
        } = my_board(cx.device);

        let led = board::led(&mut gpio2, pins.p13);
        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        iomuxc::configure(&mut pins.p34, PIN_CONFIG);
        let input = gpio2.input(pins.p34);

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        blink::spawn().unwrap();
        (Shared {}, Local { led, poller, input })
    }

    #[task(local = [led, input])]
    async fn blink(cx: blink::Context) {
        let mut count = 0u32;
        loop {
            // button grounds pin so low is pressed
            if !cx.local.input.is_set() {
                Systick::delay(100.millis()).await;
                cx.local.led.toggle();
                if count % 5 == 0 {
                    log::info!("Button is pressed! The count is {count}");
                }
            } else {
                Systick::delay(100.millis()).await;
            }
            count = count.wrapping_add(1);
        }
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
