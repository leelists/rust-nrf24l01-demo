use embedded_hal::blocking::delay::DelayMs;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::prelude::*;
use esp_idf_hal::{gpio, spi};
use std::thread;
use std::time::Duration;

use esp_idf_sys::{self};

use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::spi::config::V02Type;
use esp_idf_hal::spi::SPI2;

use nrf24_rs::config::{DataPipe, DataRate, EncodingScheme, NrfConfig, PALevel, PayloadSize};
use nrf24_rs::Nrf24l01;

mod nrfirq;

struct MyDelay {}

impl DelayMs<u8> for MyDelay {
    fn delay_ms(&mut self, ms: u8) {
        Ets.delay_ms(ms);
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
        .channel(98)
        .addr_width(5)
        .payload_size(PayloadSize::Dynamic)
        .data_rate(DataRate::R250Kbps)
        .pa_level(PALevel::Max)
        .crc_encoding_scheme(Some(EncodingScheme::R2Bytes));

    println!("{:?}", nrf_config);

    let spi = peripherals.spi2;

    let sclk :OuputPin = pins.gpio15.into_output().unwrap();
    let sdo = pins.gpio4.into_output().unwrap();
    let sdi = Some(pins.gpio12.into_input().unwrap());
    let cs = Option::<gpio::Gpio2<gpio::Output>>::None;

    let spi_pins = spi::Pins { sclk, sdo, sdi, cs };

    let ce = pins.gpio14.into_output().unwrap();
    let ncs = pins.gpio2.into_output().unwrap();

    let spi_config = spi::config::Config {
        baudrate: Hertz(1_000_000),
        data_mode: V02Type(embedded_hal::spi::MODE_0).into(),
        // write_only: false,
        // dma: Dma::Disabled,
    };

    let spi = spi::Master::<SPI2, _, _, _, _>::new(spi, spi_pins, spi_config);

    let d1 = &mut MyDelay {};
    let mut buf = vec![0_u8; 32];

    match spi {
        Ok(spi) => {
            match Nrf24l01::new(spi, ce, ncs, d1, nrf_config) {
                Ok(mut nrf_chip) => {
                    if nrf_chip.is_connected().unwrap() {
                        println!("Chip is connected.");
                    }
                    nrf_chip.set_retries((5, 15)).expect("retries");

                    nrf_chip.open_reading_pipe(DataPipe::DP1, b"MainN").expect("open_reading_pipe");
                    nrf_chip.reset_status().expect("reset");
                    nrf_chip.start_listening().expect("listening");

                    nrfirq::isr_init().expect("irq");

                    loop {
                        if !nrfirq::isr_wait() {
                            continue;
                        }
                        while let Ok(Some(pipe)) = nrf_chip.data_available_on_pipe() {
                            match pipe {
                                DataPipe::DP1 => {
                                    let len = nrf_chip.read(&mut buf).expect("read");
                                    let b = &buf[..len];
                                    println!("{}:{:?}", len, b);
                                }
                                _ => { println!("data on pipe {:?}", pipe); }
                            }
                        }
                        // reset irq
                        nrf_chip.reset_status().expect("rst");
                    }
                }
                Err(e) => {
                    println!("unable to init Nrf24l01 device {:?}", e);
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
