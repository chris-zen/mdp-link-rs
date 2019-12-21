#![no_main]
#![no_std]

#![allow(unused_imports)]
#![allow(dead_code)]

use cortex_m_rt::entry;

#[allow(unused_imports)]
//use panic_halt;
use panic_semihosting as _;

use cortex_m_semihosting::{dbg, hprintln, heprintln};

use core::fmt::Write;
use nb::block;

use embedded_hal::timer::CountDown;

use nrf52840_hal as hal;
use hal::timer::{TimerExt, Timer};
use hal::clocks::{ClocksExt, Clocks};
use hal::{Uarte, target::UARTE0};

use nrf52840_mdk::Leds;

use nrf52_radio::Radio;
use nrf52_radio::radio::RadioExt;
use nrf52_radio::tx_power::TxPower;
use nrf52_radio::mode::Mode;
use nrf52_radio::frequency::Frequency;
use nrf52_radio::logical_address::LogicalAddress;
use nrf52_radio::rx_addresses::RX_ADDRESS_ALL;
use nrf52_radio::base_address::BaseAddresses;

use nrf52_esb::{Esb, RxConfig, TxConfig, RxPacket};
use nrf52_esb::protocol::Protocol as EsbProtocol;
use nrf52840_mdk::{leds_welcome, Board};

const LED_INTERVAL: u32 = 1_000_000;


#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    drop(board.uart_daplink.write_str("Initialising ...\n"));

    let mut timer = board.TIMER0.constrain();

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

    let rx_config = RxConfig::default().with_skip_ack(true);

    drop(board.uart_daplink.write_str("Starting ...\n"));

    board.leds.green.on();
    board.leds.blue.off();
    timer.start(LED_INTERVAL);

    loop {
        if let Err(error) = esb.start_rx(rx_config) {
            board.leds.green.off();
            board.leds.red.on();
            drop(board.uart_daplink.write_fmt(format_args!("Error: {:?}\n", error)));
        }
        else {
            board.leds.green.on();
            board.leds.red.off();
            if let Err(error) = block!(esb.wait_rx()) {
                board.leds.green.off();
                board.leds.red.on();
                drop(board.uart_daplink.write_fmt(format_args!("Error: {:?}\n", error)));
            }
            else {
                board.leds.blue.invert();
                let packet = esb.get_last_received_packet().unwrap();
                let buf = esb.get_rx_buffer();
                print_packet(&packet, buf, &mut board.uart_daplink);
            }
        }

        if let Ok(()) = timer.wait() {
            board.leds.blue.invert();
            timer.start(LED_INTERVAL);
        }
    }
}

fn print_packet(packet: &RxPacket, buf: &[u8], uarte: &mut Uarte<UARTE0>) {
    let header = ((buf[0] as u16) << 8) | (buf[1] as u16);
    let buf = &buf[2..];
    let no_ack = if packet.no_ack { 1 } else { 0 };
    drop(uarte.write_fmt(format_args!("[{} {:02} {} {} {:016b}] ",
                                           packet.address.value(),
                                           packet.length,
                                           packet.pid,
                                           no_ack,
                                           header)));
    for b in buf.iter() {
        // TODO optimize
        drop(uarte.write_fmt(format_args!("{:02x} ", *b)));
    }
    drop(uarte.write_char('\n'));
}
