#![no_main]
#![no_std]

#![allow(unused_imports)]
#![allow(dead_code)]

use cortex_m_rt::entry;

#[allow(unused_imports)]
use panic_halt;
use core::fmt::Write;
use nb::block;

//use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

use nrf52840_hal as hal;
//use hal::prelude::*;
use hal::timer::{TimerExt, Timer};
use hal::clocks::{ClocksExt, Clocks};

mod nrf52840_mdk;
use nrf52840_mdk::Leds;

use nrf52_radio::radio::RadioExt;
use nrf52_radio::tx_power::TxPower;
use nrf52_radio::mode::Mode;
use nrf52_radio::frequency::Frequency;
use nrf52_radio::rx_addresses::RX_ADDRESS_ALL;
use nrf52_radio::base_address::BaseAddresses;

use nrf52_radio_esb::Esb;
use nrf52_radio_esb::protocol::Protocol as EsbProtocol;
use crate::nrf52840_mdk::Board;

use cortex_m_semihosting::{dbg, hprintln, heprintln};

#[entry]
fn main() -> ! {
    let mut board = nrf52840_mdk::Board::take().unwrap();
    let mut timer = board.TIMER0.constrain();

    drop(board.uart_daplink.write_str("Initialising ...\n"));

    leds_welcome(&mut board.leds, &mut timer);

    let clocks = board.CLOCK.constrain().enable_ext_hfosc();

    let mut buffer = [0x00u8; 48];

    let radio = board.RADIO.constrain(&clocks);
    radio
        .set_tx_power(TxPower::ZerodBm)
        .set_mode(Mode::Nrf2Mbit)
        .set_frequency(Frequency::from_2400mhz_channel(78))
        .set_base_addresses(BaseAddresses::from_same_four_bytes([0xa0, 0xb1, 0xc2, 0xd3]))
        .set_prefixes([0xe1, 0xe0, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7])
        .set_rx_addresses(RX_ADDRESS_ALL)
        .enable_power();

    let esb = Esb::new(radio, EsbProtocol::fixed_payload_length(32));
    esb.set_crc_16bits();

//    hprintln!("pcfn0={:08x}", esb.radio.radio.pcnf0.read().bits()).unwrap();
//    hprintln!("pcfn1={:08x}", esb.radio.radio.pcnf1.read().bits()).unwrap();

    drop(board.uart_daplink.write_str("Listening ...\n"));

    board.leds.red.on();

    esb.radio.enable_rx(&mut buffer);
    block!(esb.radio.wait_idle()).unwrap();
    board.leds.red.invert();

    esb.radio.start_rx(&mut buffer);
    board.leds.red.invert();

    loop {
        match esb.radio.wait_packet_received() {
            Ok(()) => {
                board.leds.blue.invert();
                if esb.radio.is_crc_ok() {
                    board.leds.red.invert();
                    for b in buffer.iter() {
                        drop(board.uart_daplink.write_fmt(format_args!("{:02x} ", *b)));
                    }
                    drop(board.uart_daplink.write_char('\n'));
                }
                else {
                    drop(board.uart_daplink.write_fmt(format_args!("rxmatch={:x} rxcrc={:x} ",
                        esb.radio.radio.rxmatch.read().bits(),
                        esb.radio.radio.rxcrc.read().bits(),
                    )));
                }
                esb.radio.start_rx(&mut buffer);
            },
            _ => {
            }
        };
    }
}

fn leds_welcome<T>(leds: &mut Leds, timer: &mut Timer<T>)
    where
        T: TimerExt,
{
    let wait_interval = 50_000;
    for _ in 0..5 {
        leds.red.on();
        delay(timer, wait_interval);
        leds.blue.on();
        delay(timer, wait_interval);
        leds.red.off();
        delay(timer, wait_interval);
        leds.green.on();
        delay(timer, wait_interval);
        leds.blue.off();
        delay(timer, wait_interval);
        leds.red.on();
        delay(timer, wait_interval);
        leds.green.off();
    }
    leds.red.off();
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
    where
        T: TimerExt,
{
    timer.start(cycles);
    drop(block!(timer.wait()));
}

