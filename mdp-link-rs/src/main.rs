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
use nrf52_radio::Radio;

#[entry]
fn main() -> ! {
    let mut board = nrf52840_mdk::Board::take().unwrap();
    let mut timer = board.TIMER0.constrain();

    drop(board.uart_daplink.write_str("Initialising ...\n"));

    leds_welcome(&mut board.leds, &mut timer);

    let clocks = board.CLOCK.constrain().enable_ext_hfosc();

    let radio = Radio::new(board.RADIO, &clocks);
    radio
        .set_tx_power(TxPower::ZerodBm)
        .set_mode(Mode::Nrf2Mbit)
        .set_frequency(Frequency::from_2400mhz_channel(78))
        .set_base_addresses(BaseAddresses::from_same_four_bytes([0xa0, 0xb1, 0xc2, 0xd3]))
        .set_prefixes([0xe1, 0xe0, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7])
        .set_rx_addresses(RX_ADDRESS_ALL)
        .enable_power();

    let mut esb = Esb::new(radio, EsbProtocol::fixed_payload_length(32));
    esb.set_crc_16bits();

//    hprintln!("pcfn0={:08x}", esb.radio.radio.pcnf0.read().bits()).unwrap();
//    hprintln!("pcfn1={:08x}", esb.radio.radio.pcnf1.read().bits()).unwrap();

    drop(board.uart_daplink.write_str("Listening ...\n"));

    board.leds.red.on();

    let mut buffer1 = [0x00u8; 34];
    let mut buffer2 = [0x00u8; 34];

    drop(esb.radio.swap_buffer(Some(&mut buffer1)));

    let mut next_buffer: Option<&mut [u8]> = Some(&mut buffer2);

    esb.start_receive().unwrap();

    loop {
        match esb.wait_receive() {
            Ok(()) => {
                board.leds.blue.invert();
                board.leds.red.invert();
                next_buffer = esb.radio.swap_buffer(next_buffer.take());
                let mut buf_iter = esb.radio.get_buffer().iter();
                let len = buf_iter.next().unwrap();
                let pid_noack = buf_iter.next().unwrap();
                let pid = pid_noack >> 1;
                let noack = pid_noack & 0x01;
                drop(board.uart_daplink.write_fmt(format_args!("[{:02x} {} {}] ", len, pid, noack & 0x01)));
                for b in buf_iter {
                    drop(board.uart_daplink.write_fmt(format_args!("{:02x} ", *b)));
                }
                drop(board.uart_daplink.write_char('\n'));
//                    drop(board.uart_daplink.write_fmt(format_args!("rxmatch={:x} rxcrc={:x} ",
//                        esb.radio.radio.rxmatch.read().bits(),
//                        esb.radio.radio.rxcrc.read().bits(),
//                    )));
                esb.start_receive().unwrap();
            },
            Err(nb::Error::Other(esb_error)) => panic!("{:?}", esb_error),
            Err(nb::Error::WouldBlock) => {}
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

