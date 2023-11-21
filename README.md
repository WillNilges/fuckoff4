# fuckoff4
Oh my god make it stop

<p align="center">
  <img height="300px" src="https://github.com/WillNilges/fuckoff4/assets/42927786/d2b6d828-ceba-49d1-a27a-3e2e9aa53e85" alt="FuckOff4 Prototype (hopefully)">
</p>

FuckOff4 is the fourth rendition of a piece of digital signage employed
at [Computer Science House](https://csh.rit.edu) to advertise events happening in
special rooms.

The project consists of two parts: The Hardware, and The Proxy.

## Hardware

<p align="center">
  <img height="300px" src="https://github.com/WillNilges/fuckoff4/assets/42927786/7638e7aa-a8a2-4c88-be0f-557f66865085" alt="FuckOff4 Prototype (hopefully)">
</p>

The Hardware consists of an ESP32 and a 1602 LCD display mated via a protoboard.
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

## The Proxy

Fill out the .env file. Then,

```
podman build . --tag fuckoff4-proxy
podman run --rm -e .env --name fuckoff4-proxy fuckoff4-proxy
```
