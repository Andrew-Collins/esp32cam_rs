use anyhow::Result;

use esp_idf_hal::peripherals::Peripherals;
use espcam::espcam::Camera;

fn main() -> Result<()> {
    // esp_idf_svc::sys::link_patches();
    // esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // Camera
    let camera = Camera::new(
        peripherals.pins.gpio32, //PWDN
        peripherals.pins.gpio15, // XCLK
        peripherals.pins.gpio2, // D0
        peripherals.pins.gpio14, // D1
        peripherals.pins.gpio35, // D2
        peripherals.pins.gpio12, // D3
        peripherals.pins.gpio27, // D4
        peripherals.pins.gpio33, // D5
        peripherals.pins.gpio34, // D6
        peripherals.pins.gpio39, // D7
        peripherals.pins.gpio18, // VSYNC
        peripherals.pins.gpio36, // HREF
        peripherals.pins.gpio26, // PCLK
        peripherals.pins.gpio22, // SDIOD
        peripherals.pins.gpio23, // SDIOC
        esp_idf_sys::camera::pixformat_t_PIXFORMAT_RGB565,
        esp_idf_sys::camera::framesize_t_FRAMESIZE_240X240,
    )
    .unwrap();

    loop {
        let framebuffer = camera.get_framebuffer();

        if let Some(framebuffer) = framebuffer {
            println!("Got framebuffer!");
            println!("width: {}", framebuffer.width());
            println!("height: {}", framebuffer.height());
            println!("len: {}", framebuffer.data().len());
            println!("format: {}", framebuffer.format());

            std::thread::sleep(std::time::Duration::from_millis(1000));
        } else {
            log::info!("no framebuffer");
        }
    }
}
