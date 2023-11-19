use embedded_hal::blocking::delay::DelayMs;
use esp_idf_hal::{
    delay::Ets,
    gpio::*,
    peripherals::Peripherals
};

use esp_idf_hal::delay::FreeRtos;

use hd44780_driver::{HD44780, DisplayMode, Cursor, CursorBlink, Display};

use embedded_svc::{
    wifi::{AuthMethod, ClientConfiguration, Configuration},
    http::{client::Client as HttpClient, Method},
    io::Read,
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi}, 
    http::client::EspHttpConnection,
};

use log::info;

pub mod config;
use crate::config::{SSID, PASSWORD, PROXY_ROUTE, HZ};

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

    let lcd_register = PinDriver::output(peripherals.pins.gpio13)?;
    let lcd_enable = PinDriver::output(peripherals.pins.gpio12)?;
    
    let lcd_d4 = PinDriver::output(peripherals.pins.gpio14)?;
    let lcd_d5 = PinDriver::output(peripherals.pins.gpio27)?;
    let lcd_d6 = PinDriver::output(peripherals.pins.gpio26)?;
    let lcd_d7 = PinDriver::output(peripherals.pins.gpio25)?;

    let mut lcd = HD44780::new_4bit(
        lcd_register,
        lcd_enable,
        lcd_d4,
        lcd_d5,
        lcd_d6,
        lcd_d7,
        &mut Ets,
    ).unwrap();
    
    // Set up the display
    let _ = lcd.reset(&mut Ets);
    let _ = lcd.clear(&mut Ets);
    let _ = lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Invisible,
            cursor_blink: CursorBlink::Off,
        },
        &mut Ets
    );

    let _ = lcd.write_str("Connecting...", &mut Ets);

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    connect_wifi(&mut wifi)?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    let _ = lcd.write_str("Setting up...", &mut Ets);

    loop {
        let proxy_response = query_proxy(PROXY_ROUTE);
        let _ = lcd.reset(&mut Ets);
        lcd.clear(&mut Ets);
        lcd.write_str("Fetch new status", &mut Ets);
        FreeRtos::delay_ms(1000);
        let _ = lcd.reset(&mut Ets);
        lcd.clear(&mut Ets);

        match proxy_response {
            Ok(d) => {
                println!("Setting display: {}", d);
                let display_text: Vec<&str> = d.split("\n").collect();
                lcd.write_str(&display_text[0], &mut Ets);
                lcd.set_cursor_pos(40, &mut Ets);
                lcd.write_str(&display_text[1], &mut Ets);
            },
            _ => {
                lcd.write_str("Error connecting to\nproxy server.", &mut Ets);
            }
        }
        FreeRtos::delay_ms(HZ);
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
            let mut offset = 0;
            let mut total = 0;
            let mut reader = response;
            let mut ex_size = 0;
            loop {
                if let Ok(size) = Read::read(&mut reader, &mut buf[offset..]) {
                    if size == 0 {
                        break;
                    }
                    ex_size = size;
                    total += size;
                }
            }

            let size_plus_offset = ex_size + offset; 
            match str::from_utf8(&buf[..size_plus_offset]) {
                Ok(text) => {
                    offset = 0;
                    Ok(text.to_string())
                },
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
