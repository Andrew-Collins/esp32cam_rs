use core::panic;

use anyhow::Result;

use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_idf_hal::delay::{self};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;
use esp_idf_sys::camera;
use espcam::espcam::{Camera, FrameBuffer};
use image::{ImageBuffer, ImageFormat, Rgb};

#[derive(Default)]
pub struct DummyTimesource();

impl TimeSource for DummyTimesource {
    // In theory you could use the RTC of the rp2040 here, if you had
    // any external time synchronizing device.
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

fn framebuffer_to_img(framebuffer: FrameBuffer<'_>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let data = framebuffer.data();
    ImageBuffer::from_fn(
        framebuffer.width() as u32,
        framebuffer.height() as u32,
        |x, y| match framebuffer.format() {
            camera::pixformat_t_PIXFORMAT_RGB565 => {
                let pix_addr = (x + y * framebuffer.width() as u32) as usize * 2;
                let raw_pixel = u16::from_be_bytes([data[pix_addr], data[pix_addr + 1]]);

                let decoded = rgb565::Rgb565::unpack_565(raw_pixel);

                Rgb([decoded.0, decoded.1, decoded.2])
            }

            camera::pixformat_t_PIXFORMAT_GRAYSCALE => {
                let pix_addr = (x + y * framebuffer.width() as u32) as usize;
                let raw_pixel = data[pix_addr];

                Rgb([raw_pixel, raw_pixel, raw_pixel])
            }

            _ => {
                panic!("unsupported format");
            }
        },
    )
}

fn main() -> Result<()> {
    // esp_idf_svc::sys::link_patches();
    // esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // Camera
    let camera = Camera::new(
        peripherals.pins.gpio32, //PWDN
        peripherals.pins.gpio15, // XCLK
        peripherals.pins.gpio2,  // D0
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

    // SD
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio4; // CLK
    let mosi = peripherals.pins.gpio21; // CMD
    let miso = peripherals.pins.gpio13; // DAT0
    let cs = peripherals.pins.gpio19; // DAT3

    // configuring the spi interface
    let config = config::Config::new().baudrate(26.MHz().into());

    let spi_dev = SpiDeviceDriver::new_single(
        spi,
        sclk,
        mosi,
        Some(miso),
        Some(cs),
        &SpiDriverConfig::new(),
        &config,
    )?;

    let sdcard = SdCard::new(spi_dev, delay::FreeRtos);
    let mut volume_mgr = VolumeManager::new(sdcard, DummyTimesource::default());

    println!("Init SD card controller and retrieve card size...");
    let sd_size = volume_mgr.device().num_bytes().unwrap();
    println!("card size is {} bytes\r\n", sd_size);

    let mut volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let mut root_dir = volume0.open_root_dir().unwrap();

    loop {
        let framebuffer = camera.get_framebuffer();

        if let Some(framebuffer) = framebuffer {
            println!("Got framebuffer!");
            println!("width: {}", framebuffer.width());
            println!("height: {}", framebuffer.height());
            println!("len: {}", framebuffer.data().len());
            println!("format: {}", framebuffer.format());
            let im = framebuffer_to_img(framebuffer);
            let mut c = std::io::Cursor::new(Vec::new());
            im.write_to(&mut c, ImageFormat::Png).unwrap();
            let mut my_file = root_dir
                .open_file_in_dir("curr.png", Mode::ReadWriteCreate)
                .unwrap();
            let _ = my_file.write(&c.into_inner());
        } else {
            log::info!("no framebuffer");
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
