use esp_idf_hal::delay::{FreeRtos, BLOCK};
use esp_idf_hal::i2c::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    const LCD_ADDRESS: u8 = 0x27; // Address depends on hardware, see link below

    // Create a I2C instance, needs to implement embedded_hal::blocking::i2c::Write, this
    // particular uses the arduino_hal crate for avr microcontrollers like the arduinos.
    /*
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI, //
        pins.a4.into_pull_up_input(), // use respective pins
        pins.a5.into_pull_up_input(),
        50000,
    );
    let mut delay = arduino_hal::Delay::new();
    */

    let peripherals = Peripherals::take()?;
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio6;

    println!("Starting I2C test");

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    Ok(())
}
