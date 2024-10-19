//! The starter code slowly blinks the LED and sets up
//! USB logging. It periodically logs messages over USB.
//!
//! This template uses [RTIC v2](https://rtic.rs/2/book/en/)
//! for structuring the application.

#![no_std]
#![no_main]

use teensy4_panic as _;
use bsp::board;
use bsp::{
    hal::{gpio, iomuxc, timer::Blocking},
    pins,
};
use teensy4_bsp as bsp;

use imxrt_log as logging;

/// input on Pin 34
type Input =  gpio::Input<pins::t41::P34>;


struct Rotary {
    s1: gpio::Input<pins::t41::P10>,
    s2: gpio::Input<pins::t41::P11>,
    key: gpio::Input<pins::t41::P12>,
}

    /// pull up value should be twice input in impedance
const PIN_PULLUP: iomuxc::Config = iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pullup47k));


#[bsp::rt::entry]
fn main() -> ! { 
    let board::Resources {
        pit,
        mut gpio2,
        mut pins,
        usb,
        ..
    } = board::t41(board::instances());

    let led = board::led(&mut gpio2, pins.p13);
    let mut poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

    iomuxc::configure(&mut pins.p34, PIN_PULLUP);
    let input = gpio2.input(pins.p34);

    // let rotary = Rotary {
    //     s1: gpio2.input(pins.p10),
    //     s2: gpio2.input(pins.p11),
    //     key: gpio2.input(pins.p12),
    // };

    let mut delay = Blocking::<_, { board::PERCLK_FREQUENCY }>::from_pit(pit.0);
    let mut count = 0;

    loop {
        poller.poll();
        // button grounds pin so low is pressed
        if !input.is_set() {
            led.set();
            if count % 5 == 0 {
                log::info!("Button is pressed! The count is {count}");
            }
        } else {
            led.clear();
        }
        delay.block_ms(50);
        count = count + 1;
    }

}

