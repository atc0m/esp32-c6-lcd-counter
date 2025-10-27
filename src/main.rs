use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver, SpiDriverConfig, Dma};
use esp_idf_hal::spi::config::Config;
use esp_idf_hal::units::Hertz;
use esp_idf_hal::gpio::AnyIOPin;
use ws2812_esp32_rmt_driver::driver::color::{LedPixelColor, LedPixelColorGrb24};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;
//use std::thread::sleep;
//use std::time::Duration;
//use rand::Rng;
use mipidsi::{Builder, models::ST7789, options::{ColorOrder, Orientation, Rotation, ColorInversion}};
use mipidsi::interface::SpiInterface;
use embedded_graphics::{
    primitives::{Rectangle, PrimitiveStyle, PrimitiveStyleBuilder},
    pixelcolor::Rgb565,
    draw_target::DrawTarget,
    prelude::*,
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    text::{Alignment, Text},
};
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

// Draw large digits using rectangles (seven-segment style)
fn draw_digit<D: DrawTarget<Color = Rgb565>>(display: &mut D, digit: u8, x: i32, y: i32) where D::Error: std::fmt::Debug {
    let segments = [
        // Top
        Rectangle::new(Point::new(x + 2, y), Size::new(16, 3)),
        // Top right
        Rectangle::new(Point::new(x + 18, y + 2), Size::new(3, 18)),
        // Bottom right
        Rectangle::new(Point::new(x + 18, y + 22), Size::new(3, 18)),
        // Bottom
        Rectangle::new(Point::new(x + 2, y + 40), Size::new(16, 3)),
        // Bottom left
        Rectangle::new(Point::new(x, y + 22), Size::new(3, 18)),
        // Top left
        Rectangle::new(Point::new(x, y + 2), Size::new(3, 18)),
        // Middle
        Rectangle::new(Point::new(x + 2, y + 20), Size::new(16, 3)),
    ];
    
    // Which segments are lit for each digit
    let digit_segments = [
        [1, 1, 1, 1, 1, 1, 0], // 0
        [0, 1, 1, 0, 0, 0, 0], // 1
        [1, 1, 0, 1, 1, 0, 1], // 2
        [1, 1, 1, 1, 0, 0, 1], // 3
        [0, 1, 1, 0, 0, 1, 1], // 4
        [1, 0, 1, 1, 0, 1, 1], // 5
        [1, 0, 1, 1, 1, 1, 1], // 6
        [1, 1, 1, 0, 0, 0, 0], // 7
        [1, 1, 1, 1, 1, 1, 1], // 8
        [1, 1, 1, 1, 0, 1, 1], // 9
    ];
    
    let pattern = &digit_segments[digit as usize];
    
    for (i, segment) in segments.iter().enumerate() {
        let color = if pattern[i] == 1 {
            Rgb565::GREEN
        } else {
            Rgb565::new(2, 2, 2) // Dim gray for "off" segments
        };
        segment
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(display)
            .unwrap();
    }
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

    let (r, g, b) = Color::Red.rgb();
    update_led_colour(r, g, b, &mut driver);

    // Display
    let rst_pin = peripherals.pins.gpio21;
    let dc_pin = peripherals.pins.gpio15;
    let scl_pin = peripherals.pins.gpio7;
    let mosi_pin = peripherals.pins.gpio6;
    let cs_pin = peripherals.pins.gpio14;

    let backlight_pin = peripherals.pins.gpio22;  // <-- UPDATE THIS PIN NUMBER

    // Configure the backlight pin and turn it on
    let mut backlight = PinDriver::output(backlight_pin).unwrap();
    backlight.set_high().unwrap(); // Turn on backlight

    // Configure the Reset (RST) pin as an output
    let mut rst = PinDriver::output(rst_pin).unwrap();
    // Configure the Data/Command (DC) pin as an output
    let dc = PinDriver::output(dc_pin).unwrap();

    // Manual reset of display
    rst.set_low().unwrap();
    FreeRtos::delay_ms(10);
    rst.set_high().unwrap();
    FreeRtos::delay_ms(120); // Give display time to initialize

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        scl_pin, // SCL
        mosi_pin, // MOSI
        Option::<AnyIOPin>::None,                    // MISO - not used for the display
        &SpiDriverConfig::new().dma(Dma::Auto(4096)),
    ).unwrap();

    let spi_config = Config::new().baudrate(Hertz(40_000_000));

    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        Some(cs_pin), // CS
        &spi_config,
    ).unwrap();

    static mut BUFFER: [u8; 4096] = [0; 4096];

    let di = SpiInterface::new(spi_device, dc, unsafe { &mut BUFFER });

    // Create the display driver instance using the mipidsi Builder
    let mut display = Builder::new(ST7789, di)
        .display_size(172, 320)
        .display_offset(34, 0)
        .color_order(ColorOrder::Rgb)
        .reset_pin(rst)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation {
            mirrored: false,
            rotation: Rotation::Deg90, // For landscape
        })
        .init(&mut FreeRtos)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let display_size = display.size();
    log::info!("Display size: {:?}", display_size);

    // Draw a red border around the entire display
    let border = Rectangle::new(
        Point::new(0, 0),
        Size::new(display_size.width, display_size.height)
    );
    border.into_styled(
        PrimitiveStyleBuilder::new()
            .stroke_color(Rgb565::RED)
            .stroke_width(2)
            .fill_color(Rgb565::BLACK)
            .build()
        )
        .draw(&mut display)
        .unwrap();

    // Draw corner markers to identify orientation
    // Top-left: Green
    Rectangle::new(Point::new(0, 0), Size::new(20, 20))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut display)
        .unwrap();

    // Top-right: Blue
    Rectangle::new(
        Point::new(display_size.width as i32 - 20, 0), 
        Size::new(20, 20)
    )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
        .draw(&mut display)
        .unwrap();

    // Bottom-left: Yellow
    Rectangle::new(
        Point::new(0, display_size.height as i32 - 20), 
        Size::new(20, 20)
    )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::YELLOW))
        .draw(&mut display)
        .unwrap();

    // Bottom-right: Magenta
    Rectangle::new(
        Point::new(
            display_size.width as i32 - 20, 
            display_size.height as i32 - 20
        ), 
        Size::new(20, 20)
    )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::MAGENTA))
        .draw(&mut display)
        .unwrap();

    let (r, g, b) = Color::Green.rgb();
    update_led_colour(r, g, b, &mut driver);
    log::info!("Done");

    log::info!("Display initialized successfully!");


    // Display a number
    let mut number = 0;

    // Keep the program running - this is important!
    loop {
        number = number + 1;
        FreeRtos::delay_ms(1000);
        log::info!("Running...");
        // Use it to display a number
        let digits = [
            (number / 1000) % 10,
            (number / 100) % 10,
            (number / 10) % 10,
            number % 10,
        ];

        for (i, &digit) in digits.iter().enumerate() {
            draw_digit(&mut display, digit as u8, 50 + (i as i32 * 25), 50);
        }
    }

}
