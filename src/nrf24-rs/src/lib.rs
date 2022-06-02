//! This crate provides a platform agnostic Rust driver for the nRF24L01+ single chip 2.4 GHz
//! transceiver by Nordic Semiconduct for communicating data wirelessly using the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! # Usage
//!
//! This crate can be used by adding `nrf24-rs` to your dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! nrf24-rs = "0.1"
//! ```
//!
//! # Overview
//!
//! # Example: Sending data
//! This simple example will send a simple "Hello world" message.
//! ```rust
//! use panic_halt as _;
//!
//! use atmega168_hal as hal;
//! use hal::prelude::*;
//! use hal::spi;
//! use nrf24_rs::config::{NrfConfig, PALevel};
//! use nrf24_rs::{Nrf24l01, SPI_MODE};
//!
//! #[atmega168_hal::entry]
//! fn main() -> ! {
//!     // Take peripherals
//!     let dp = hal::pac::Peripherals::take().unwrap();
//!
//!     // Initialize the different pins
//!     let mut portb = dp.PORTB.split();
//!     let ncs = portb.pb2.into_output(&mut portb.ddr);
//!     let mosi = portb.pb3.into_output(&mut portb.ddr);
//!     let miso = portb.pb4.into_pull_up_input(&mut portb.ddr);
//!     let sclk = portb.pb5.into_output(&mut portb.ddr);
//!
//!     // Initialize SPI
//!     let settings = spi::Settings {
//!         data_order: spi::DataOrder::MostSignificantFirst,
//!         clock: spi::SerialClockRate::OscfOver4,
//!         mode: SPI_MODE, // SPI Mode defined in this crate
//!     };
//!     let (spi, ncs) = spi::Spi::new(dp.SPI, sclk, mosi, miso, ncs, settings);
//!
//!     let mut delay = hal::delay::Delay::<hal::clock::MHz16>::new();
//!
//!     let message = b"Hello world!"; // The message we will be sending
//!
//!     // Setup some configuration values
//!     let config = NrfConfig::default()
//!         .channel(8)
//!         .pa_level(PALevel::Min)
//!         // We will use a payload size the size of our message
//!         .payload_size(message.len());
//!
//!     // Initialize the chip
//!     let mut nrf_chip = Nrf24l01::New(spi, ce, ncs, &mut delay, config).unwrap();
//!     if !nrf_chip.is_connected().unwrap() {
//!         panic!("Chip is not connected.");
//!     }
//!
//!     // Open a writing pipe on address "Node1".
//!     // The listener will have to open a reading pipe with the same address
//!     // in order to recieve this message.
//!     nrf.open_writing_pipe(b"Node1").unwrap();
//!
//!     // Keep trying to send the message
//!     while let Err(e) = nrf.write(&mut delay, &message) {
//!         // Something went wrong while writing, try again in 50ms
//!         delay.delay_ms(50u16);
//!     }
//!
//!     // Message should now successfully have been sent!
//!     loop {}
//! }
//! ```
//!
//!
//! # Example: Reading data
//! This simple example will read a "Hello world" message.
//! ```rust
//! use panic_halt as _;
//!
//! use atmega168_hal as hal;
//! use hal::prelude::*;
//! use hal::spi;
//! use nrf24_rs::config::{NrfConfig, PALevel, DataPipe};
//! use nrf24_rs::{Nrf24l01, SPI_MODE};
//!
//! #[atmega168_hal::entry]
//! fn main() -> ! {
//!     // Take peripherals
//!     let dp = hal::pac::Peripherals::take().unwrap();
//!
//!     // Initialize the different pins
//!     let mut portb = dp.PORTB.split();
//!     let ncs = portb.pb2.into_output(&mut portb.ddr);
//!     let mosi = portb.pb3.into_output(&mut portb.ddr);
//!     let miso = portb.pb4.into_pull_up_input(&mut portb.ddr);
//!     let sclk = portb.pb5.into_output(&mut portb.ddr);
//!
//!     // Initialize SPI
//!     let settings = spi::Settings {
//!         data_order: spi::DataOrder::MostSignificantFirst,
//!         clock: spi::SerialClockRate::OscfOver4,
//!         mode: SPI_MODE, // SPI Mode defined in this crate
//!     };
//!     let (spi, ncs) = spi::Spi::new(dp.SPI, sclk, mosi, miso, ncs, settings);
//!
//!     let mut delay = hal::delay::Delay::<hal::clock::MHz16>::new();
//!
//!     // Setup some configuration values
//!     let config = NrfConfig::default()
//!         .channel(8)
//!         .pa_level(PALevel::Min)
//!         // We will use a payload size the size of our message
//!         .payload_size(b"Hello world!".len());
//!
//!     // Initialize the chip
//!     let mut nrf_chip = Nrf24l01::New(spi, ce, ncs, &mut delay, config).unwrap();
//!     if !nrf_chip.is_connected().unwrap() {
//!         panic!("Chip is not connected.");
//!     }
//!
//!     // Open reading pipe 0 with address "Node1".
//!     // The sender will have to open its writing pipe with the same address
//!     // in order to transmit this message successfully.
//!     nrf_chip.open_reading_pipe(DataPipe::DP0, b"Node1").unwrap();
//!     // Set the chip in RX mode
//!     nrf_chip.start_listening().unwrap();
//!
//!     // Keep checking if there is any data available to read
//!     while !nrf_chip.data_available().unwrap() {
//!         // No data availble, wait 50ms, then check again
//!         delay.delay_ms(50u16);
//!     }
//!     // Now there is some data availble to read
//!
//!     // Initialize empty buffer
//!     let mut buffer = [0; b"Hello world!".len()];
//!     nrf_chip.read(&mut buffer).unwrap();
//!
//!     assert_eq!(buffer, b"Hello world!");
//!
//!     loop {}
//! }
//! ```
//!
//! # Feature-flags
//!
//! - **micro-fmt:** provides a `uDebug` implementation from the [ufmt crate](https://docs.rs/ufmt) for all public structs and enums.
#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts
)]
#![no_std]
extern crate embedded_hal as hal;
use hal::spi;

pub mod config;
mod error;
mod nrf24;
mod register_acces;
pub mod status;

pub use crate::error::TransferError;
pub use crate::nrf24::Nrf24l01;

/// SPI mode. Use this when initializing the SPI instance.
pub const SPI_MODE: spi::Mode = spi::MODE_0;
/// Max size in bytes of a single payload to be sent or recieved.
pub const MAX_PAYLOAD_SIZE: u8 = 32;
