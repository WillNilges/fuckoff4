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
    http::{client::Client as HttpClient},
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
        //let _ = lcd.reset(&mut Ets);
        //let _ = lcd.clear(&mut Ets);

        // TODO: use shift_display to scroll the title, probably have to re-draw
        // the time each frame. Need a timestamp for last refresh, and check
        // that every frame, refreshing if the time since that hits like 30
        // or whatever
        
        // FIXME: The display has a buffer of some sort that the driver doesnt
        // really account for. I think the max we can do is 42.
        // Maybe I will chunk it into 40 characters.
        
        match proxy_response {
            Ok(d) => {
                println!("Setting display: {}", d);
                let display_text: Vec<&str> = d.split("\n").collect();
                let time = format!("{: <16}", &display_text[1]);

                if display_text[0].len() > 16 {
                    for i in (0..display_text[0].len()-12).step_by(4) {
                        let mut t: String = display_text[0].chars().into_iter().skip(i).take(16).collect();
                        t = format!("{: <16}", t);
                        let _ = lcd.set_cursor_pos(0, &mut Ets);
                        let _ = lcd.write_str(&t, &mut Ets);
                        let _ = lcd.set_cursor_pos(40, &mut Ets);
                        let _ = lcd.write_str(&time, &mut Ets);
                        FreeRtos::delay_ms(1000);
                    }
                    FreeRtos::delay_ms(1000);
                } else {
                    let text = format!("{: <16}", &display_text[0]);
                    let _ = lcd.set_cursor_pos(0, &mut Ets);
                    let _ = lcd.write_str(&text, &mut Ets);
                    let _ = lcd.set_cursor_pos(40, &mut Ets);
                    let _ = lcd.write_str(&time, &mut Ets);
                    FreeRtos::delay_ms(HZ);
                }

                //                                        | It breaks here.
                // Willards test event that is really long and has a very longwinded explaination
                //let _ = lcd.write_str("Willards test event that is really longxx", &mut Ets);
                //let _ = lcd.write_str("abcdefghijklmnopqrstuvwxyz 1234567890 ss", &mut Ets);
                /*
                let _ = lcd.write_str(&display_text[0], &mut Ets);
                let _ = lcd.set_cursor_pos(40, &mut Ets);
                let _ = lcd.write_str(&display_text[1], &mut Ets);
                */

                // For now, I am going to kludge this. I will scroll the title three times
                // then refresh. If it's longer than 16 characters, it will take
                // at least 8 seconds to scroll, and that's plenty of time between
                // refreshes.
                /*if display_text[0].len() > 16 {
                    display_text[0].chars().for_each(|_| {
                        let _ = lcd.shift_display(hd44780_driver::Direction::Left, &mut Ets);
                        FreeRtos::delay_ms(1000);
                    }); 
                } else {
                    FreeRtos::delay_ms(HZ);
                }*/
            },
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
