#![no_std]
#![no_main]

//SPI TODO
//https://docs.rs/imxrt-hal/latest/imxrt_hal/lpspi/index.html

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {

    use leadscrew::*;
    use teensy4_bsp::hal::gpio::Trigger;
    use teensy4_panic as _;
    use bsp::{board, pins};
    use bsp::hal::{gpio, iomuxc};
    use board::t41 as my_board;
    use teensy4_bsp as bsp;
    use tm1637_embedded_hal::asynch::TM1637;
    use tm1637_embedded_hal::Brightness;
    use imxrt_log as logging;
    use rtic_monotonics::systick::{Systick, *};

    // we need to import this trait to get our i2c writes and reads
    // use embedded_hal::i2c::I2c;

    // PIN assignemnts
    // 4-digit display
    type Pin4digitDio = pins::t41::P18;
    type Pin4digitClk = pins::t41::P19;

    // NOT USING
    /// input on Pin 34
    // type Input =  gpio::Input<pins::t41::P34>;


    struct Rotary {
        s1: gpio::Input<pins::t41::P10>,
        s2: gpio::Input<pins::t41::P11>,
        key: gpio::Input<pins::t41::P12>,
    }

    /// pull up value should be twice input in impedance
    const PIN_CONFIG_PULLUP: iomuxc::Config = iomuxc::Config::modify().set_pull_keeper(Some(iomuxc::PullKeeper::Pullup47k));
    const PIN_CONFIG_OPENDRAIN: iomuxc::Config = iomuxc::Config::modify().set_open_drain(iomuxc::OpenDrain::Enabled);

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
        seg7: TM1637<gpio::Output<Pin4digitClk>, gpio::Output<Pin4digitDio>, Tm1637Delay>,
        // lpspi3: my_board::Lpspi3<teensy4_bsp::pins::common::P1, teensy4_bsp::pins::common::P0>,
        enc_a: gpio::Input<pins::t41::P23>,
        enc_b: gpio::Input<pins::t41::P21>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            // lpi2c1,
            mut gpio2,
            mut gpio1,
            pins,
            // mut lpspi3,
            usb,
            ..
        } = my_board(cx.device);

        // mutable pin value assignments
        let pin_rotary_s1 = pins.p10;
        let pin_rotary_s2 = pins.p11;
        let mut pin_rotary_key = pins.p12;
        let pin_led = pins.p13;
        let mut pin_4digit_dio = pins.p18;
        let mut pin_4digit_clk = pins.p19;
        let mut pin_encoder_a = pins.p23;
        let mut pin_encoder_b = pins.p21;


        iomuxc::configure(&mut pin_encoder_a, PIN_CONFIG_PULLUP);
        iomuxc::configure(&mut pin_encoder_b, PIN_CONFIG_PULLUP);


        // low-level interrupts at bsp::ral::interrupt
        // board-level interrupts at  bsp::interrupt

        let enc_a = gpio1.input(pin_encoder_a);
        let enc_b = gpio1.input(pin_encoder_b);
        
        gpio1.set_interrupt(&enc_a, Some(Trigger::EitherEdge));
        gpio1.set_interrupt(&enc_b, Some(Trigger::EitherEdge));

        let led = board::led(&mut gpio2, pin_led);
        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        // let mut lpspi3 = board::lpspi(
        //             lpspi3,
        //             board::LpspiPins {
        //                 sdo: pins.p26,
        //                 sdi: pins.p1,
        //                 sck: pins.p27,
        //                 pcs0: pins.p0,
        //             },
        //             1_000_000,
        //         );


        iomuxc::configure(&mut pin_rotary_key, PIN_CONFIG_PULLUP);
        let rotary = Rotary {
            s1: gpio2.input(pin_rotary_s1),
            s2: gpio2.input(pin_rotary_s2),
            key: gpio2.input(pin_rotary_key),
        };

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );


        iomuxc::configure(&mut pin_4digit_clk, PIN_CONFIG_OPENDRAIN);
        iomuxc::configure(&mut pin_4digit_dio, PIN_CONFIG_OPENDRAIN);

        let clk = gpio1.output(pin_4digit_clk);
        let dio = gpio1.output(pin_4digit_dio);
        let SEG7_NUM_POSITIONS = 4;
        let SEG7_DELAY_US = 1000;
        let brightness = Brightness::L7;
        let seg7 = TM1637::new(clk, dio, Tm1637Delay {}, brightness, SEG7_DELAY_US, SEG7_NUM_POSITIONS);


        // poll_encoder::spawn().unwrap();
        // update_seg7::spawn().unwrap();
        blink::spawn().unwrap();

        (Shared {}, Local { led, poller, rotary, seg7, enc_a, enc_b })
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
                    log::info!("Button is pressed!");
                }
            } else {
                Systick::delay(100.millis()).await;
            }
            count = count.wrapping_add(1);
        }
    }


    #[task(local=[seg7])]
    async fn update_seg7(cx: update_seg7::Context) {

        Systick::delay(1000.millis()).await;
        cx.local.seg7.init().await.unwrap();
        Systick::delay(10.millis()).await;
        cx.local.seg7.write_brightness(Brightness::L7).await.unwrap();

        let mut counter = 0;

        loop {
            let segs = [digit_byte((0 + counter) % 9),digit_byte(1), digit_byte(2), digit_byte(3)];
            cx.local.seg7.write_segments_raw(0, &segs).await.unwrap();
            counter = counter + 1;
            Systick::delay(100.millis()).await;
            log::info!("Seg 7");
        }

    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }

    // #[task(binds = GPIO1_COMBINED_16_31, local=[enc_a, enc_b])]
    // fn process_encoder(cx: process_encoder::Context) {
    //     log::info!("Encoder interrupt triggered");
    // }
    
    #[task(local=[enc_a, enc_b])]
    async fn poll_encoder(cx: poll_encoder::Context) {
        loop {
            log::info!("A: {}", cx.local.enc_a.is_set());
            log::info!("B: {}", cx.local.enc_b.is_set());
            
            Systick::delay(100.millis()).await;
        }
    }



    // we need the name of the 
     // #[task(binds = BOARD_BUTTON, local = [led, delay, button])]
    // fn button_press(cx: button_press::Context) {
     //    let delay = cx.local.delay;
     //    let led = cx.local.led;
     //    let button = cx.local.button;

     //    led.toggle();
     //    button.clear_triggered();
     //    delay.block_us(1000);
    // }
    
}
