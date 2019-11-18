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
use nrf52_radio::logical_address::LogicalAddress;
use nrf52_radio::rx_addresses::RX_ADDRESS_ALL;
use nrf52_radio::base_address::BaseAddresses;

use nrf52_radio_esb::Esb;
use nrf52_radio_esb::protocol::Protocol as EsbProtocol;
use crate::nrf52840_mdk::Board;

use cortex_m_semihosting::{dbg, hprintln, heprintln};
use nrf52_radio::Radio;

enum State {
    Sending,
    Listening
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
    let mut esb = Esb::new(radio, EsbProtocol::fixed_payload_length(32), &mut buffer1, &mut buffer2);
    esb.set_crc_16bits();

//    hprintln!("pcfn0={:08x}", esb.radio.radio.pcnf0.read().bits()).unwrap();
//    hprintln!("pcfn1={:08x}", esb.radio.radio.pcnf1.read().bits()).unwrap();

//    drop(board.uart_daplink.write_str("Listening ...\n"));
    drop(board.uart_daplink.write_str("Sending ...\n"));

    board.leds.red.on();

    // M01 asks P905 to connect
    let msg1: [u8; 34] = [33, 0,
        0x09, 0x08, 0x62, 0x6d, 0xfa, 0x5d, 0x00, 0x01, 0x5a,
        0x73, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00];

    // P905 responds connection request to M01
    let msg2: [u8; 34] = [33, 0,
        0x09, 0x0d, 0x62, 0x6d, 0xfa, 0x5d, 0x00, 0x00, 0x3e,
        0xc2, 0x3b, 0x00, 0x0f, 0x78, 0x6d, 0xf9, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00];

    let msg = &msg2;

//    esb.get_buffer_mut().copy_from_slice(msg);

//    esb.start_tx(LogicalAddress::Of0).unwrap();
    esb.start_rx().unwrap();

    let mut state = State::Listening;
    let mut retries = 1;

    loop {
        match state {
            State::Sending => {
                match esb.wait_tx() {
                    Ok(()) => {
                        board.leds.blue.invert();
                        board.leds.red.invert();

                        retries -= 1;

                        if retries == 0 {
                            drop(board.uart_daplink.write_str("Done\n"));

                            state = State::Listening;
                            esb.start_rx().unwrap();
                        }
                        else {
                            drop(board.uart_daplink.write_str("Retry\n"));
                            esb.get_buffer_mut().copy_from_slice(msg);
                            esb.start_tx(LogicalAddress::Of0).unwrap();
                        }
                    },
                    Err(nb::Error::Other(esb_error)) => panic!("{:?}", esb_error),
                    Err(nb::Error::WouldBlock) => {}
                }
            },
            State::Listening => {
                match esb.wait_rx() {
                    Ok(()) => {
                        board.leds.blue.invert();
                        board.leds.red.invert();
                        let buf_iter = esb.get_buffer().iter().skip(2);
                        let packet = esb.get_last_received_packet().unwrap();
                        let no_ack = if packet.no_ack { 1 } else { 0 };
                        drop(board.uart_daplink.write_fmt(format_args!("[{} {:02x} {} {}] ",
                                                                       packet.address.value(),
                                                                       packet.length,
                                                                       packet.pid,
                                                                       no_ack)));
                        for b in buf_iter {
                            drop(board.uart_daplink.write_fmt(format_args!("{:02x} ", *b)));
                        }
                        drop(board.uart_daplink.write_char('\n'));
                        esb.start_rx().unwrap();
                    },
                    Err(nb::Error::Other(esb_error)) => panic!("{:?}", esb_error),
                    Err(nb::Error::WouldBlock) => {}
                }
            }
        }
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

