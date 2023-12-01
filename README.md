# Project Sidegrade
Oh my god make it stop

<p align="center">
  <img height="300px" src="https://github.com/WillNilges/sidegrade/assets/42927786/5012d524-64e7-4df9-a1ba-9387a7c7d776" alt="FuckOff4 Prototype (hopefully)">
</p>


Sidegrade is the fourth rendition of a piece of digital signage employed
at [Computer Science House](https://csh.rit.edu) to advertise events happening in
special rooms.

The project consists of two parts: The Hardware, and The Proxy.

## Hardware

The Hardware consists of an [ESP32](https://www.amazon.com/dp/B09QW6Y7KY?psc=1&ref=ppx_yo2ov_dt_b_product_details)
and a [2004 display](https://www.amazon.com/dp/B0C1G9GBRZ?psc=1&ref=ppx_yo2ov_dt_b_product_details). The ESP32 has
no headers, and the 2004 comes with an i2c adapter board. Using the provided Amazon links at time of writing it
costs about $34 to build. It will also require soldering supplies, four 90° male pin headers, some wire, and four
M3*16 machine screws.

Solder the following pins together, with the ESP32 facing away from the LCD. Leave some slack so that you can manipulate the two halves of the case later (ESP32 mounts in the back, 2004 Display mounts in the front)

| 2004 I2C Board | GND | VCC | SDA | SCL |
|----------------|-----|-----|-----|-----|
| ESP32          | GND | VIN | D33 | D32 |

Print the case. It shouldn't matter what filament is used.

Place the ESP32 on the pegs on the bottom case. Place the LCD on top of that and get the wires situated. Then, put the top case on and use the screws to fasten the top case, LCD, and bottom case together.

The firmware is written in Rust using tools from the excellent [esp-rs project](https://github.com/esp-rs).
It connects to a WiFi network, and periodically queries the proxy for text to display on the screen. It
is chiefly used to display events from the Google Calendar API. The first row is used to display the name of
the event, scrolling if necessary. The second row is used to display the time until the event, or the time
left in the event.

## Proxy

The Proxy is a simple webserver written in Rust using Actix Web. It serves the routes, unenecrypted, that the
Hardware checks to get event information. The location of interest (i.e. where the device is installed) is
configurable.

### Routes

**`/locations/<location>/event`**

Queries the Google Calendar API for events, then looks for the next event that is happening, parsing the response
and formatting it for the display.
Example:
```
History
In 00:15:23
```

# Development

Install Rust and follow the guide available in [The Rust on ESP Book](https://esp-rs.github.io/book/installation/index.html).

Specifically, [this page](https://esp-rs.github.io/book/installation/riscv-and-xtensa.html) is how you install for Xtensa platforms.

In the root of this project, run:
```
cargo install espup
espup install
cargo install ldproxy
cargo install espflash
```

These pages will probably be useful:
- https://github.com/esp-rs/esp-idf-hal
    - The Hardware Abstraction Layer library for ESP. Implements `embedded-hal`.
- https://github.com/esp-rs/esp-idf-svc
    - Wrappers for abstracting some of the more complex featuees of the ESP32, such as WiFi and HTTP requests.
 
# Deployment

## The Hardware

Plug in your ESP32, then run:

```
cargo run
```

If you just want to get serial output, you can do:

```
cargo espflash monitor
```

## The Proxy

Fill out the .env file. Then,

```
podman build . --tag fuckoff4-proxy
podman run --rm -e .env --name fuckoff4-proxy fuckoff4-proxy
```
