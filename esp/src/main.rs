use esp_idf_hal::{delay::{FreeRtos, Ets}, i2c::*, peripherals::Peripherals};

use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};

use embedded_svc::{
    http::client::Client as HttpClient,
    io::Read,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::EspHttpConnection,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};


use esp_idf_hal::prelude::*;

use log::info;

pub mod config;
use crate::config::{HZ, PASSWORD, PROXY_ROUTE, SSID, I2C_ADDR};

use core::str;

use anyhow::bail;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    println!("Booting Fuckoff4...");
    println!("Waiting for display...");

    let i2c = peripherals.i2c1;
    let sda = peripherals.pins.gpio13;
    let scl = peripherals.pins.gpio12;

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

    let _ = lcd.write_str("Connecting...", &mut Ets);

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    connect_wifi(&mut wifi)?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    let _ = lcd.reset(&mut Ets);
    let _ = lcd.clear(&mut Ets);

    loop {
        let proxy_response = query_proxy(PROXY_ROUTE);
        //let _ = lcd.reset(&mut Ets);
        //let _ = lcd.clear(&mut Ets);

        // TODO: use shift_display to scroll the title, probably have to re-draw
        // the time each frame. Need a timestamp for last refresh, and check
        // that every frame, refreshing if the time since that hits like 30
        // or whatever

        // FIXME: The display has a buffer of some sort that the driver doesnt
        // really account for. I think the max we can do is 42.
        // Maybe I will chunk it into 40 characters.
        
        #[derive(Clone)]
        enum LCDRow {
            First = 0x00,
            Second = 0x40,
            Third = 0x14,
            Fourth = 0x54
        }

        match proxy_response {
            Ok(d) => {
                println!("Setting display: {}", d);
                let display_text: Vec<&str> = d.split('\n').collect();

                let mut l_pos = vec![0, 0, 0, 0];
                let row = vec![LCDRow::First, LCDRow::Second, LCDRow::Third, LCDRow::Fourth]; 
                
                loop {
                    for (idx, line) in display_text.iter().enumerate() {
                        // If the line length is >20, then step
                        // the line
                        if line.len() > 20 {
                            let mut t: String = line.chars().skip(l_pos[idx]).take(20).collect();
                            t = format!("{: <20}", t);
                            let _ = lcd.set_cursor_pos(row[idx].clone() as u8, &mut Ets);
                            let _ = lcd.write_str(&t, &mut Ets);
                            
                            if l_pos[idx] > line.len() - 16 {
                                l_pos[idx] = 0;
                            } else {
                                l_pos[idx] += 4;
                            }
                        } else {
                            let t = format!("{: <20}", &line);
                            let _ = lcd.set_cursor_pos(row[idx].clone() as u8, &mut Ets);
                            let _ = lcd.write_str(&t, &mut Ets);
                        }
                    }
                    FreeRtos::delay_ms(1000);
                }
            }
            _ => {
                let _ = lcd.write_str("Error connecting to\nproxy server.", &mut Ets);
            }
        }
    }
}

fn query_proxy(url: impl AsRef<str>) -> anyhow::Result<String> {
    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let request = client.get(url.as_ref())?;
    let response = request.submit()?;
    let status = response.status();

    println!("Status: {}", status);

    match status {
        200..=299 => {
            let mut buf = [0_u8; 256];
            let offset = 0;
            let mut reader = response;
            let mut ex_size = 0;
            loop {
                if let Ok(size) = Read::read(&mut reader, &mut buf[offset..]) {
                    if size == 0 {
                        break;
                    }
                    ex_size = size;
                }
            }

            let size_plus_offset = ex_size + offset;
            match str::from_utf8(&buf[..size_plus_offset]) {
                Ok(text) => Ok(text.to_string()),
                Err(_error) => {
                    bail!("Fuck")
                }
            }
        }
        _ => bail!("Unexpected response code: {}", status),
    }
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.into(),
        channel: None,
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}

// TODO: Make lcd object
//fn step_line( display_text: String, t_pos: usize, s_pos: usize) {
//    let mut t: String = display_text.chars().skip(t_pos).take(20).collect();
//    t = format!("{: <20}", t);
//    let _ = lcd.set_cursor_pos(t_pos, &mut Ets);
//    let _ = lcd.write_str(&t, &mut Ets);
//}
