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

    // we need to import this trait to get our i2c writes and reads
    use embedded_hal::i2c::I2c;

    use imxrt_log as logging;

    // Using Teensy 4.1
    use board::t41 as my_board;


    use rtic_monotonics::systick::{Systick, *};

    // NOT USING
    // use tm1637_embedded_hal::{asynch::TM1637, demo::asynch::Demo};

    // NOT USING
    /// input on Pin 34
    // type Input =  gpio::Input<pins::t41::P34>;
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

    struct Rotary {
        s1: gpio::Input<pins::t41::P10>,
        s2: gpio::Input<pins::t41::P11>,
        key: gpio::Input<pins::t41::P12>,
    }

    /// pull up value should be twice input in impedance
    const PIN_PULLUP: iomuxc::Config = iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pullup47k));

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
        // input: Input,
        /// the rotary switch
        rotary: Rotary,
        seg7: bsp::board::Lpi2c1,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            lpi2c1,
            mut gpio2,
            mut pins,
            usb,
            ..
        } = my_board(cx.device);

        let led = board::led(&mut gpio2, pins.p13);
        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        //lpi2c1 has pin 19 clock line
        //pin 18 data line

        let seg7: bsp::board::Lpi2c1 = board::lpi2c(lpi2c1, pins.p19, pins.p18, board::Lpi2cClockSpeed::KHz100);

        // iomuxc::configure(&mut pins.p34, PIN_PULLUP);
        // let input = gpio2.input(pins.p34);


        iomuxc::configure(&mut pins.p12, PIN_PULLUP);
        let rotary = Rotary {
            s1: gpio2.input(pins.p10),
            s2: gpio2.input(pins.p11),
            key: gpio2.input(pins.p12),
        };

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        blink::spawn().unwrap();
        update_seg7::spawn().unwrap();
        (Shared {}, Local { led, poller, rotary, seg7 })
    }

    #[task(local = [led, rotary])]
    async fn blink(cx: blink::Context) {
        let mut count = 0u32;
        loop {
            // button grounds pin so low is pressed
            if !cx.local.rotary.key.is_set() {
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

    #[task(local=[seg7])]
    async fn update_seg7(cx: update_seg7::Context) {
        const SLAVE_ADDR: u8 = 0x48;  // The device's I2C address

        // these look right
        const DISPLAY_REGISTERS: [u8; 4] = [0x68, 0x6A, 0x6C, 0x6E];  // Register addresses for each digit

        cx.local.seg7.set_controller_enable(true);
        // Data command setting (Mode command)
        Systick::delay(1000.millis()).await;
        
        // Data command setting (Mode command)
        if let Err(e) = cx.local.seg7.write(SLAVE_ADDR, &[0x48]) {
            log::error!("Failed to set data command mode. Error: {:?}", e);
        }
        // Display control: ON, max brightness
        if let Err(e) = cx.local.seg7.write(SLAVE_ADDR, &[0x71]) {
            log::error!("Failed to set max brightness. Error: {:?}", e);
        }

        let digits = [1, 2, 3, 4];
        loop {
            for (pos, &digit) in digits.iter().enumerate() {
                    // Select the register for this digit
                    // 
                if let Err(e) = cx.local.seg7.write(SLAVE_ADDR, &[DISPLAY_REGISTERS[pos]]) {
                    log::error!("Failed to write register. Error: {:?}", e);
                }
                if let Err(e) = cx.local.seg7.write(SLAVE_ADDR, &[DIGITS[digit]]) {
                    log::error!("Failed to write digit. Error: {:?}", e);
                }
            }
            // Wait for 1 second before updating again
            Systick::delay(100.millis()).await;
            log::info!("Display write complete");
        }
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
