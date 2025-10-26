use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver, SpiDriverConfig, Dma};
use esp_idf_hal::spi::config::Config;
use ws2812_esp32_rmt_driver::driver::color::{LedPixelColor, LedPixelColorGrb24};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;
use display_interface_spi::SPIInterfaceNoCS;
use mipidsi::{Builder, models::ST7789, options::{ColorOrder, Orientation}};
use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_hal::delay::FreeRtos;

enum Color {
    Red,
    Orange,
    Green,
}

impl Color {
    fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Red => (255, 0, 0),
            Color::Orange => (255, 165, 0),
            Color::Green => (0, 255, 0),
        }
    }
}

fn update_led_colour(r: u8, g: u8, b: u8, driver: &mut Ws2812Esp32RmtDriver) {
    let color = LedPixelColorGrb24::new_with_rgb(g, r, b);
    let pixel: [u8; 3] = color.as_ref().try_into().unwrap();
    driver.write_blocking(pixel.clone().into_iter()).unwrap();
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Initialising...");
    // Get peripherals
    let peripherals = Peripherals::take().unwrap();

    // LED
    let led_pin = peripherals.pins.gpio8;
    let channel = peripherals.rmt.channel0;
    let mut driver = Ws2812Esp32RmtDriver::new(channel, led_pin).unwrap();

    // Display
    let rst_pin = peripherals.pins.gpio21;
    let dc_pin = peripherals.pins.gpio15;
    let scl_pin = peripherals.pins.gpio7;
    let mosi_pin = peripherals.pins.gpio6;
    let cs_pin = peripherals.pins.gpio14;

    // Configure the Reset (RST) pin as an output
    let rst = PinDriver::output(rst_pin).unwrap();
    // Configure the Data/Command (DC) pin as an output
    let dc = PinDriver::output(dc_pin).unwrap();

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        scl_pin, // SCL
        mosi_pin, // MOSI
        None,                    // MISO - not used for the display
        &SpiDriverConfig::new().dma(Dma::Auto(1024)),
    ).unwrap();

    let spi_config = esp_idf_hal::spi::config::Config::new().baudrate(40_i32.MHz().into());

    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        Some(cd_pin), // CS
        &spi_config,
    ).unwrap();

    // assert_eq!(pixel, [0, 30, 0]);

    let (r, g, b) = Color::Red.rgb();
    update_led_colour(r, g, b, &mut driver);
    log::info!("Done");

}
