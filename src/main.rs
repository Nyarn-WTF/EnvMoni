#![no_std]
#![no_main]

use panic_halt as _;
use wio_terminal as wio;

use core::fmt::Write;
use scd30::scd30::Scd30;
use wio::entry;
use wio::hal::clock::GenericClockController;
use wio::hal::gpio::*;
use wio::hal::sercom::*;
use wio::pac::Peripherals;
use wio::prelude::*;
use wio::UART;
use wio_terminal::hal::delay::Delay;
use wio_terminal::pac::CorePeripherals;

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

    let mut pins = wio::Pins::new(peripherals.PORT);

    // UARTドライバオブジェクトを初期化する
    let uart = UART {
        tx: pins.txd,
        rx: pins.rxd
    };

    let mut serial = uart.init(
        &mut clocks,
        115200.hz(),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        &mut pins.port,
    );

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
        pins.i2c1_sda.into_pad(&mut pins.port),
        pins.i2c1_scl.into_pad(&mut pins.port),
    );

    let mut delay = Delay::new(core.SYST, &mut clocks);

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

    loop {
        let mut data = scd.read().unwrap().unwrap();
        writeln!(&mut serial, "CO2: {}, TEMP: {}, HUM: {}\r\n", data.co2, data.temperature, data.humidity).unwrap();
        delay.delay_ms(5000u16);
    }
}
