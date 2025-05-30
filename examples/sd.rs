use std::ascii::AsciiExt;

use anyhow::Result;
use embedded_hal::spi::MODE_0;

use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_idf_hal::delay::{self};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;
use espcam::espcam::Camera;

#[derive(Default)]
pub struct DummyTimesource();

const DEFAULT_SLEEP_S: u64 = 86370; // 24 hours - 30s

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

fn main() -> Result<()> {
    // esp_idf_svc::sys::link_patches();
    // esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // Camera
    let camera = Camera::new(
        peripherals.pins.gpio32, //PWDN
        peripherals.pins.gpio0,  // XCLK
        peripherals.pins.gpio5,  // D0
        peripherals.pins.gpio18, // D1
        peripherals.pins.gpio19, // D2
        peripherals.pins.gpio21, // D3
        peripherals.pins.gpio36, // D4
        peripherals.pins.gpio39, // D5
        peripherals.pins.gpio34, // D6
        peripherals.pins.gpio35, // D7
        peripherals.pins.gpio25, // VSYNC
        peripherals.pins.gpio23, // HREF
        peripherals.pins.gpio22, // PCLK
        peripherals.pins.gpio26, // SDIOD
        peripherals.pins.gpio27, // SDIOC
        esp_idf_sys::camera::pixformat_t_PIXFORMAT_JPEG,
        esp_idf_sys::camera::framesize_t_FRAMESIZE_UXGA,
    )
    .unwrap();

    // SD
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio14; // CLK
    let mosi = peripherals.pins.gpio15; // CMD
    let miso = peripherals.pins.gpio2; // DAT0
    let cs = peripherals.pins.gpio13; // DAT3

    // configuring the spi interface
    let config = config::Config::new()
        .baudrate(400.kHz().into())
        .data_mode(MODE_0);

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
    let mut i = 0;
    let _ = root_dir.iterate_dir(|entry| {
        let ext = entry.name.extension().to_ascii_lowercase();
        if ext == "jpeg".as_bytes() || ext == "jpg".as_bytes() {
            i += 1
        }
    });
    // Start at one as user will probably prefer that
    i += 1;
    println!("Im ind: {}", i);

    let mut s = String::new();
    if let Ok(mut f) = root_dir.open_file_in_dir("CONFIG.TXT", Mode::ReadOnly) {
        let mut buffer = [0u8; 16];
        let n = f.read(&mut buffer).unwrap();
        for b in buffer.iter().take(n) {
            let ch = char::from(*b);
            if ch.is_ascii_graphic() {
                s.push(ch);
                // print!("{}", ch);
            }
        }
        let _ = f.close();
    }
    let timer_s = s.parse::<u64>().unwrap_or(DEFAULT_SLEEP_S);
    println!("Sleep: {}s", timer_s);

    for _ in 0..100 {
        camera.get_framebuffer();
    }

    // loop {
    camera.get_framebuffer();
    let framebuffer = camera.get_framebuffer();

    if let Some(framebuffer) = framebuffer {
        println!("Got framebuffer!");
        println!("width: {}", framebuffer.width());
        println!("height: {}", framebuffer.height());
        println!("len: {}", framebuffer.data().len());
        println!("format: {}", framebuffer.format());
        let name = format!("{}.jpg", i);
        i += 1;
        let mut my_file = root_dir
            .open_file_in_dir(name.as_str(), Mode::ReadWriteCreate)
            .unwrap();
        let _ = my_file.write(framebuffer.data());
    } else {
        println!("no framebuffer");
    }

    println!("Waiting");
    unsafe {
        // esp_idf_hal::sys::sleep(10);
        esp_idf_hal::sys::esp_deep_sleep(timer_s * 1000 * 1000);
    }
    // std::thread::sleep(std::time::Duration::from_millis(10000));
    // }
}
