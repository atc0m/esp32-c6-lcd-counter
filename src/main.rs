use esp_idf_hal::peripherals::Peripherals;
use ws2812_esp32_rmt_driver::driver::color::{LedPixelColor, LedPixelColorGrb24};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Get peripherals
    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio8;
    let channel = peripherals.rmt.channel0;

    let mut driver = Ws2812Esp32RmtDriver::new(channel, led_pin).unwrap();
    // assert_eq!(pixel, [0, 30, 0]);

    loop {
        let mut r: u8 = rand::thread_rng().gen();
        let mut g: u8 = rand::thread_rng().gen();
        let mut b: u8 = rand::thread_rng().gen();

        let color = LedPixelColorGrb24::new_with_rgb(r, g, b);
        let pixel: [u8; 3] = color.as_ref().try_into().unwrap();
        driver.write_blocking(pixel.clone().into_iter()).unwrap();
        sleep(Duration::from_millis(100));
    }
    

    log::info!("do you see the red colour?");
}
