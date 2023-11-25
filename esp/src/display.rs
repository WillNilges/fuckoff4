use std::sync::{Arc, Mutex};
use embedded_hal::blocking::i2c;
use esp_idf_hal::{
    delay::{Ets, FreeRtos},
    gpio::{InputPin, OutputPin},
    i2c::*,
    peripheral::Peripheral,
    prelude::*,
};
use hd44780_driver::{
    bus::{DataBus, I2CBus},
    Cursor, CursorBlink, Display, DisplayMode, HD44780,
};

use crate::config::I2C_ADDR;

pub struct SidegradeDisplay<B: DataBus> {
    pub lcd: HD44780<B>,
    pub text: Vec<String>,
}

#[derive(Clone)]
enum LCDRow {
    First = 0x00,
    Second = 0x40,
    Third = 0x14,
    Fourth = 0x54,
}

impl<B: DataBus> SidegradeDisplay<B> {
    // Gross, yet convenient methods
    pub fn wipe(&mut self) {
        self.lcd.reset(&mut Ets).unwrap();
        self.lcd.clear(&mut Ets).unwrap();
    }

    pub fn write(&mut self, string: &str) {
        self.lcd.write_str(string, &mut Ets).unwrap();
    }

    pub fn flash(&mut self, count: u32, hz: u32) {
        for _ in 0..count {
            let _ = self.lcd.set_display_mode(
                DisplayMode {
                    display: Display::Off,
                    cursor_visibility: Cursor::Invisible,
                    cursor_blink: CursorBlink::Off,
                },
                &mut Ets,
            );
            FreeRtos::delay_us(hz);
            let _ = self.lcd.set_display_mode(
                DisplayMode {
                    display: Display::Off,
                    cursor_visibility: Cursor::Invisible,
                    cursor_blink: CursorBlink::Off,
                },
                &mut Ets,
            );
        }
    }

    pub fn run(&mut self, m: Arc<Mutex<Vec<String>>>) -> anyhow::Result<()> {
        // Create a position vector and a finished vector for each line
        let mut l_pos = [0; 4];
        let mut l_fin = [false; 4];
        let row = vec![LCDRow::First, LCDRow::Second, LCDRow::Third, LCDRow::Fourth];

        loop {
            for (idx, line) in self.text.iter().enumerate() {
                // If the line length is >20, then step the line
                if line.len() > 20 {
                    let mut t: String = line.chars().skip(l_pos[idx]).take(20).collect();
                    t = format!("{: <20}", t);
                    let _ = self.lcd.set_cursor_pos(row[idx].clone() as u8, &mut Ets);
                    let _ = self.lcd.write_str(&t, &mut Ets);

                    if l_pos[idx] > line.len() - 16 {
                        l_pos[idx] = 0;
                        l_fin[idx] = true;
                    } else {
                        l_pos[idx] += 4;
                    }
                } else {
                    let t = format!("{: <20}", &line);
                    let _ = self.lcd.set_cursor_pos(row[idx].clone() as u8, &mut Ets);
                    let _ = self.lcd.write_str(&t, &mut Ets);
                    l_fin[idx] = true;
                }
            }
            FreeRtos::delay_ms(1000);
            if l_fin.iter().all(|&x| x) {
                {
                    let num = m.lock().unwrap();
                    self.text = (*num.clone()).to_vec();
                }
            }
        }
    }
}

impl<'d, I2C: i2c::Write> SidegradeDisplay<I2CBus<I2C>> {
    pub fn new_i2c<I: I2c>(
        i2c: impl Peripheral<P = I> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> anyhow::Result<SidegradeDisplay<I2CBus<I2cDriver<'d>>>> {
        let config = I2cConfig::new().baudrate(100.kHz().into());
        let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;
        let mut lcd = HD44780::new_i2c(i2c_driver, I2C_ADDR, &mut Ets).unwrap();

        // Set up the display
        let _ = lcd.reset(&mut Ets);
        let _ = lcd.clear(&mut Ets);
        let _ = lcd.set_display_mode(
            DisplayMode {
                display: Display::On,
                cursor_visibility: Cursor::Invisible,
                cursor_blink: CursorBlink::Off,
            },
            &mut Ets,
        );

        Ok(SidegradeDisplay {
            lcd,
            text: vec![String::new(); 4],
        })
    }
}
