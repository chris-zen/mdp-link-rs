#![no_main]
#![no_std]

use cortex_m_rt::entry;

#[allow(unused_imports)]
//use panic_halt;
use panic_semihosting as _;

use cortex_m_semihosting::{dbg, hprintln, heprintln};

use core::fmt::Write;

use nrf52840_hal as hal;
//use hal::prelude::*;
use hal::timer::{TimerExt, Timer};
use hal::clocks::ClocksExt;

use nrf52840_mdk::{Board, leds_welcome};

use nrf52_radio::Radio;
use nrf52_radio::tx_power::TxPower;
use nrf52_radio::mode::Mode;
use nrf52_radio::frequency::Frequency;
use nrf52_radio::rx_addresses::RX_ADDRESS_ALL;
use nrf52_radio::base_address::BaseAddresses;

use nrf52_esb::{Esb, protocol::Protocol as EsbProtocol};

use mdp_protocols::m01;


#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();
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

    let _leds = &mut board.leds;

//    let mut p905 = p905::Protocol::new(esb, &mut board.uart_daplink);
    let mut m01 = m01::Protocol::new(esb, &mut board.uart_daplink);

    loop {
//        p905.run();
        m01.run()
    }
}
