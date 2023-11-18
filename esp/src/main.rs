use esp_idf_hal::{
    delay::Ets,
    gpio::*,
    peripherals::Peripherals
};

use log::error;
use chrono::{DateTime, Utc};

use hd44780_driver::{HD44780, DisplayMode, Cursor, CursorBlink, Display};

use embedded_svc::{
    wifi::{AuthMethod, ClientConfiguration, Configuration},
    http::{client::Client as HttpClient, Method},
    utils::io,
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi}, 
    http::client::EspHttpConnection,
};

use log::info;

pub mod config;
use crate::config::{SSID, PASSWORD, API_KEY, CALENDAR_ID};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take()?;

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    connect_wifi(&mut wifi)?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    println!("Holy shit it's willard. I'm doing GPIO things.");

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

    // Create HTTP(S) client
    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    // GET
    get_request(&mut client)?;

    Ok(())
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

fn get_request(client: &mut HttpClient<EspHttpConnection>) -> anyhow::Result<()> {
    // Get the current UTC time
    let utc: DateTime<Utc> = Utc::now();

    // Format the time as ISO 8601
    let iso_time = utc.to_rfc3339();

    let headers = [
        ("maxResults", "10"),
        ("orderBy", "startTime"),
        ("showDeleted", "false"),
        ("singleEvents", "true"),
        ("timeMin", &iso_time),
        ("fields", "items(location, start, end, summary, description"),
        ("key", API_KEY),
    ];

    let url = format!("https://www.googleapis.com/calendar/v3/calendars/{}/events", CALENDAR_ID);

    // Send request
    //
    // Note: If you don't want to pass in any headers, you can also use `client.get(url, headers)`.
    let request = client.request(Method::Get, &url, &headers)?;
    info!("-> GET {}", url);
    let mut response = request.submit()?;

    // Process response
    let status = response.status();
    info!("<- {}", status);
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    info!("Read {} bytes", bytes_read);
    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => info!(
            "Response body (truncated to {} bytes): {:?}",
            buf.len(),
            body_string
        ),
        Err(e) => error!("Error decoding response body: {}", e),
    };

    // Drain the remaining response bytes
    while response.read(&mut buf)? > 0 {}

    Ok(())
    
}
