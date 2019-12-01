#![no_main]
#![no_std]

#![allow(unused_imports)]
#![allow(dead_code)]

use cortex_m_rt::entry;

#[allow(unused_imports)]
//use panic_halt;
use panic_semihosting as _;

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

mod mdp;

use nrf52_radio::radio::RadioExt;
use nrf52_radio::tx_power::TxPower;
use nrf52_radio::mode::Mode;
use nrf52_radio::frequency::Frequency;
use nrf52_radio::logical_address::LogicalAddress;
use nrf52_radio::rx_addresses::RX_ADDRESS_ALL;
use nrf52_radio::base_address::BaseAddresses;

use nrf52_radio_esb::{Esb, RxConfig, TxConfig};
use nrf52_radio_esb::protocol::Protocol as EsbProtocol;
use crate::nrf52840_mdk::Board;

use cortex_m_semihosting::{dbg, hprintln, heprintln};
use nrf52_radio::Radio;

enum State {
    Sending,
    Sniffing
}

#[entry]
fn main() -> ! {
    let mut board = nrf52840_mdk::Board::take().unwrap();
    let mut timer = board.TIMER0.constrain();

    drop(board.uart_daplink.write_str("Initialising ...\n"));

    leds_welcome(&mut board.leds, &mut timer);

    let clocks = board.CLOCK.constrain().enable_ext_hfosc();

    let radio = Radio::new(board.RADIO, &clocks);
    radio
        .set_tx_power(TxPower::Pos8dBm)
        .set_mode(Mode::Nrf2Mbit)
        .set_frequency(Frequency::from_2400mhz_channel(78))
        .set_base_addresses(BaseAddresses::from_same_four_bytes([0xa0, 0xb1, 0xc2, 0xd3]))
        .set_prefixes([0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7])
        .set_rx_addresses(RX_ADDRESS_ALL)
        .enable_power();

    let mut buffer1 = [0x00u8; 34];
    let mut buffer2 = [0x00u8; 34];

    // TODO EsbProtocol and buffers size must match
    let esb = Esb::new(radio, EsbProtocol::fixed_payload_length(32), &mut buffer1, &mut buffer2);
    esb.set_crc_16bits();

    drop(board.uart_daplink.write_str("Starting ...\n"));

    let leds = &mut board.leds;

//    red_led.on();

//    let mut p905 = mdp::p905::Protocol::new(esb, &mut board.uart_daplink);
    let mut m01 = mdp::m01::Protocol::new(esb, &mut board.uart_daplink);

    loop {
//        p905.run();
//        p905.send_pairing_request(leds);
//        p905.sniffer();
        m01.run()
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

