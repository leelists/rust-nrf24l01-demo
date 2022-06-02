use std::thread;
use std::time::Duration;
use esp_idf_hal::prelude::*;
use esp_idf_hal::{gpio, spi};
use esp_idf_hal::delay::FreeRtos;
use embedded_hal::blocking::delay::DelayMs;

use esp_idf_sys::{self};

use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::spi::{SPI2};
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayUs;
use esp_idf_hal::spi::config::V02Type;

use nrf24_rs::config::{DataRate, EncodingScheme, NrfConfig, PALevel, PayloadSize};
use nrf24_rs::{Nrf24l01};

use nrf24_rs::TransferError::*;

struct MyDelay {}

impl DelayMs<u8> for MyDelay {
    fn delay_ms(&mut self, ms: u8) {
        FreeRtos.delay_us(ms as u32 * 1000_u32);
    }
}

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let pins = peripherals.pins;
    let nrf_config = NrfConfig::default()
        .payload_size(PayloadSize::Dynamic) // set dynamic payload size
        .channel(7)
        .addr_width(3)
        .data_rate(DataRate::R2Mbps)
        .pa_level(PALevel::Min)
        .channel(8)
        .crc_encoding_scheme(Some(EncodingScheme::R2Bytes));

    let spi = peripherals.spi2;

    let sclk = pins.gpio15.into_output()?;
    let sdo = pins.gpio12.into_input()?;
    let sdi = Some(pins.gpio4.into_output()?);
    let cs = Option::<gpio::Gpio2<gpio::Output>>::None;

    let ce = pins.gpio14.into_output().unwrap();
    let ncs = pins.gpio2.into_output().unwrap();

    let spi_config =  esp_idf_hal::spi::config::Config {
        baudrate: Hertz(1_000_000),
        data_mode: V02Type(embedded_hal::spi::MODE_0).into(),
        // <- add BitOrder here if v0.30
    };

    let spi_pins = spi::Pins { sclk, sdo, sdi, cs };
    let spi = spi::Master::<SPI2, _, _, _, _>::new(
        spi,
        spi_pins,
        spi_config,
    );

    match spi {
        Ok(spi) => {
            let mut d0 = MyDelay {};
            let d1 = &mut d0;

            match Nrf24l01::new(spi, ce, ncs, d1, nrf_config) {
                Ok(mut nrf_chip) => {
                    if !nrf_chip.is_connected().unwrap() {
                        println!("Chip is not connected.");
                        loop {}
                    }
                }
                Err(e) => {
                    println!("unable to init Nrf24l01 device");
                    match e {
                        Spi(e0) => {
                            println!("spi err:{}", e0);
                        }
                        // Pin set error
                        Pin(_) => {
                            println!("pin err");
                        }
                        // Communication error with module
                        CommunicationError(code) => {
                            println!("comm err:{}", code);
                        }
                        // Max retries reached
                        MaximumRetries=>  {
                            println!("max retries");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("spi error {}", e.code());
        }
    }

    // delay
    loop {
        thread::sleep(Duration::from_secs(1));
        println!("*");
    }
}

