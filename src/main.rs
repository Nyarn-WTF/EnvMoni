#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use core::fmt::*;
use scd30::scd30::Scd30;
use wio::entry;
use wio::hal::clock::GenericClockController;
use wio::hal::gpio::*;
use wio::hal::sercom::*;
use wio::pac::Peripherals;
use wio::prelude::*;
use wio_terminal::hal::delay::Delay;
use wio_terminal::pac::CorePeripherals;

use embedded_graphics as eg;
use eg::{fonts::*, pixelcolor::*, prelude::*, primitives::*, style::*};
use heapless::String;
use heapless::consts::*;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut delay = Delay::new(core.SYST, &mut clocks);
    let mut sets = wio::Pins::new(peripherals.PORT).split();

    let mut serial = sets
        .uart
        .init(
        &mut clocks,
        115200.hz(),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        &mut sets.port,
    );

    //LCD
    let (mut display, _backlight) = sets
        .display
        .init(
            &mut clocks,
            peripherals.SERCOM7,
            &mut peripherals.MCLK,
            &mut sets.port,
            58.mhz(),
            &mut delay,
        )
        .unwrap();

    // I2Cドライバオブジェクトを初期化する
    let gclk0 = &clocks.gclk0();
    let mut i2c: I2CMaster3<
        Sercom3Pad0<Pa17<PfD>>,
        Sercom3Pad1<Pa16<PfD>>,
    > = I2CMaster3::new(
        &clocks.sercom3_core(&gclk0).unwrap(),
        400.khz(),
        peripherals.SERCOM3,
        &mut peripherals.MCLK,
        sets.grove_i2c.sda.into_pad(&mut sets.port),
        sets.grove_i2c.scl.into_pad(&mut sets.port),
    );



    let style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::BLACK)
        .build();
    let background =
        Rectangle::new(Point::new(0, 0), Point::new(319, 239))
            .into_styled(style);
    background.draw(&mut display).unwrap();

    // Connect to sensor
    let mut scd = Scd30::new_with_address(i2c, 0x61);

    if let Err(x) = scd.set_measurement_interval(1) {
        loop {
            writeln!(&mut serial, "Err").unwrap();
        }
    }

    if let Err(x) = scd.start_measuring() {
        loop {
            writeln!(&mut serial, "Err").unwrap();
        }
    }

    let style = TextStyleBuilder::new(Font12x16)
        .text_color(Rgb565::GREEN)
        .background_color(Rgb565::BLACK)
        .build();

    loop {
        Text::new("Temperature", Point::new(10, 10))
            .into_styled(TextStyle::new(Font12x16, Rgb565::WHITE))
            .draw(&mut display).unwrap();

        Text::new("C", Point::new(220, 40))
            .into_styled(TextStyle::new(Font12x16, Rgb565::WHITE))
            .draw(&mut display).unwrap();

        Text::new("Humidity", Point::new(10, 70))
            .into_styled(TextStyle::new(Font12x16, Rgb565::WHITE))
            .draw(&mut display).unwrap();

        Text::new("%", Point::new(220, 100))
            .into_styled(TextStyle::new(Font12x16, Rgb565::WHITE))
            .draw(&mut display).unwrap();

        Text::new("CO2 Concentration", Point::new(10, 130))
            .into_styled(TextStyle::new(Font12x16, Rgb565::WHITE))
            .draw(&mut display).unwrap();

        Text::new("ppm", Point::new(220, 160))
            .into_styled(TextStyle::new(Font12x16, Rgb565::WHITE))
            .draw(&mut display).unwrap();

        let mut data = scd.read().unwrap().unwrap();
        writeln!(&mut serial, "CO2: {}, TEMP: {}, HUM: {}\r\n", data.co2, data.temperature, data.humidity).unwrap();

        let mut co2_label = String::<U256>::new();
        let mut temp_label = String::<U256>::new();
        let mut humid_label = String::<U256>::new();

        write!(&mut co2_label, "{:-2.1}", data.co2).unwrap();
        write!(&mut temp_label, "{:-2.1}", data.temperature).unwrap();
        write!(&mut humid_label, "{:-2.1}", data.humidity).unwrap();

        Text::new(&mut temp_label, Point::new(10, 40))
            .into_styled(style)
            .draw(&mut display).unwrap();

        Text::new(&mut humid_label, Point::new(10, 100))
            .into_styled(style)
            .draw(&mut display).unwrap();

        Text::new(&mut co2_label, Point::new(10, 160))
            .into_styled(style)
            .draw(&mut display).unwrap();

        delay.delay_ms(10000u16);
        background.draw(&mut display).unwrap();
    }
}
