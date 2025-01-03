
// needed for the DelayNs trait
use rtic_monotonics::systick::{Systick, *};
use embedded_hal_async::delay::DelayNs;

pub struct Tm1637Delay {}

    impl DelayNs for Tm1637Delay {
        async fn delay_ns(&mut self, x: u32) {
            Systick::delay(x.nanos()).await
        }
    }

// Segment bit positions
const SEG_A: u8 = 0b0000_0001;
const SEG_B: u8 = 0b0000_0010;
const SEG_C: u8 = 0b0000_0100;
const SEG_D: u8 = 0b0000_1000;
const SEG_E: u8 = 0b0001_0000;
const SEG_F: u8 = 0b0010_0000;
const SEG_G: u8 = 0b0100_0000;
const SEG_DP: u8 = 0b1000_0000;

// Common digit patterns
const DIGITS: [u8; 10] = [
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F,        // 0
    SEG_B | SEG_C,                                         // 1
    SEG_A | SEG_B | SEG_D | SEG_E | SEG_G,                // 2
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_G,                // 3
    SEG_B | SEG_C | SEG_F | SEG_G,                        // 4
    SEG_A | SEG_C | SEG_D | SEG_F | SEG_G,                // 5
    SEG_A | SEG_C | SEG_D | SEG_E | SEG_F | SEG_G,        // 6
    SEG_A | SEG_B | SEG_C,                                // 7
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F | SEG_G, // 8
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_F | SEG_G,        // 9
];

pub fn digit_byte(x: usize) -> u8 {
    DIGITS[x]
}
