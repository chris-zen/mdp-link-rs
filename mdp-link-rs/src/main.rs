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
use nrf52_radio::states::Disabled;
use nrf52_radio::frequency::Frequency;
use nrf52_radio::rx_addresses::RX_ADDRESS_ALL;
use nrf52_radio::base_address::BaseAddresses;

use nrf52_radio_esb::Esb;
use nrf52_radio_esb::protocol::Protocol as EsbProtocol;

#[entry]
fn main() -> ! {
    let mut board = nrf52840_mdk::Board::take().unwrap();
    let mut timer = board.TIMER0.constrain();

    leds_welcome(&mut board.leds, &mut timer);

    board.CLOCK.constrain().enable_ext_hfosc();

    let mut buffer: [u8; 64] = [0u8; 64];

    let radio = board.RADIO.constrain()
        .set_tx_power(TxPower::ZerodBm)
        .set_mode(Mode::Nrf2Mbit)
        .set_frequency(Frequency::from_2400mhz_channel(78))
        .set_base_addresses(BaseAddresses::from_same_four_bytes([0xa0, 0xb1, 0xc2, 0xd3]))
        .set_prefixes([0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7])
        .set_rx_addresses(RX_ADDRESS_ALL);

    let esb = Esb::<Disabled>::new(radio)
        .set_protocol(EsbProtocol::fixed_payload(32))
        .set_crc_16bits()
        .enable_rx(&mut buffer);

    loop {
        match board.uart_daplink.write_char('.') {
            Ok(()) => {

            },
            Err(_) => {

            }
        }

        delay(&mut timer, 1_000_000);
    }
}

fn leds_welcome<T>(leds: &mut Leds, timer: &mut Timer<T>)
    where
        T: TimerExt,
{
    let wait_interval = 100_000;
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