use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;

use hd44780_driver::{HD44780, DisplayMode, Cursor, CursorBlink, Display};


fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    //const LCD_ADDRESS: u8 = 0x27; // Address depends on hardware, see link below

    let peripherals = Peripherals::take()?;
    //let i2c = peripherals.i2c0;
    //let sda = peripherals.pins.gpio5;
    //let scl = peripherals.pins.gpio6;

    println!("Holy shit it's willard. I'm doing GPIO things.");

    //let config = I2cConfig::new().baudrate(100.kHz().into());
    //let mut i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    let mut lcd_register = PinDriver::output(peripherals.pins.gpio13)?;
    let mut lcd_enable = PinDriver::output(peripherals.pins.gpio12)?;
    
    let mut lcd_d4 = PinDriver::output(peripherals.pins.gpio14)?;
    let mut lcd_d5 = PinDriver::output(peripherals.pins.gpio27)?;
    let mut lcd_d6 = PinDriver::output(peripherals.pins.gpio26)?;
    let mut lcd_d7 = PinDriver::output(peripherals.pins.gpio25)?;

    let mut lcd = HD44780::new_4bit(
        lcd_register,
        lcd_enable,
        lcd_d4,
        lcd_d5,
        lcd_d6,
        lcd_d7,
        &mut Ets,
    ).unwrap();
    
    lcd.reset(&mut Ets);
    
    lcd.clear(&mut Ets);

    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Visible,
            cursor_blink: CursorBlink::On,
        },
        &mut Ets
    );

    lcd.write_str("Hello, world!", &mut Ets);
    

    Ok(())
}
