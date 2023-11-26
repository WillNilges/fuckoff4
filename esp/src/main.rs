use anyhow::bail;
use embedded_svc::{
    http::client::Client as HttpClient,
    io::Read,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
};

use esp_idf_hal::{
    delay::FreeRtos,
    i2c::*,
    peripherals::Peripherals,
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::EspHttpConnection,
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi},
};

use hd44780_driver::bus::I2CBus;

use log::{info, warn, error};

pub mod config;
pub mod display;

use crate::{
    config::{HZ, PASSWORD, PROXY_ROUTE, SSID},
    display::*,
};

use core::str;
use std::{sync::{Arc, Mutex}, net::ToSocketAddrs};

use futures::executor::block_on;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Booting Sidegrade...");

    let peripherals = Peripherals::take()?;

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let timer_service = EspTaskTimerService::new()?;

    // Set up display
    info!("Waiting for display...");
    let i2c = peripherals.i2c1;
    let sda = peripherals.pins.gpio13;
    let scl = peripherals.pins.gpio12;
    let mut lcd = SidegradeDisplay::<I2CBus<I2cDriver>>::new_i2c(i2c, sda, scl)?;

    // Connect to Wifi
    lcd.write("Connecting...");
    info!("Setting up Wifi...");
    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
        timer_service,
    )?;

    loop {
        info!("Connecting WiFi...");
        match block_on(connect_wifi(&mut wifi)) {
            Ok(()) => break,
            Err(e) => {
                warn!("Connection failed. Trying again.");
                lcd.write(format!("Can't connect.\n{}", e).as_str());
                // Flash the screen three times to indicate that we can't
                // Connect. Side effect of delaying 3 seconds.
                lcd.flash(3, 1000)
            },
        };
    }

    // TODO:
    // Once we're connected, print info and start API query service, as well
    // as display service
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    lcd.wipe();

    let screen_updates = Arc::new(Mutex::new(vec![String::new(); 4]));
    let lcd_screen_updates = Arc::clone(&screen_updates);
    let query_screen_updates = Arc::clone(&screen_updates);

    let lcd_thread = std::thread::Builder::new()
        .spawn(move || -> anyhow::Result<()> { lcd.run(lcd_screen_updates) });

    /*
    * I suppose this is the bonafide main thread.
    * Nominally, it will query the proxy for a new string every HZ
    * milliseconds.
    *
    * If it fails to do so because of an issue with the server (that is, it can
    * connect to the internet, but the server doesn't respond), it will update
    * the display and set it to flash, then try to re-connect.
    */
    let proxy_thread = std::thread::Builder::new().spawn(move || -> anyhow::Result<()> {
        loop {
            let proxy_response = query_proxy(PROXY_ROUTE);
            let mut num = query_screen_updates.lock().unwrap();
            match proxy_response {
                Ok(r) => {
                    *num = r.split('\n').map(String::from).collect();
                },
                Err(e) => {
                    error!("Proxy Thread Error: {}", e);
                    *num = vec!["Could not fetch updates.".to_string(), "".to_string(), "".to_string(), "".to_string()];
                },
            }
            FreeRtos::delay_ms(HZ);
        }
    });

    lcd_thread?.join().unwrap()?;
    proxy_thread?.join().unwrap()?;
    info!("Joined threads");

    loop {
        // Don't let the idle task starve and trigger warnings from the watchdog.
        FreeRtos::delay_ms(100);
    }

    //Ok(())

    /*
    loop {
        let proxy_response = query_proxy(PROXY_ROUTE)?;
        {
            let mut num = query_screen_updates.lock().unwrap();
            *num = proxy_response.split('\n').map(String::from).collect();
        }
        //FreeRtos::delay_ms(HZ);
        for _ in 0..100 {
            FreeRtos::delay_ms(100);
        }
    }
    */

}

async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.into(),
        channel: None,
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start().await?;
    info!("Wifi started");

    wifi.connect().await?;
    info!("Wifi connected");

    wifi.wait_netif_up().await?;
    info!("Wifi netif up");

    Ok(())
}

fn query_proxy(url: impl AsRef<str>) -> anyhow::Result<String> {
    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let request = client.get(url.as_ref())?;
    let response = request.submit()?;
    let status = response.status();

    info!("Proxy Query status: {}", status);

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
                Err(e) => {
                    error!("Could not decode proxy response");
                    bail!(e)
                }
            }
        }
        _ => bail!(format!("Got non-200 status: {}", status))
    }
}

