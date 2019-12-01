#![no_std]

//use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

use nb::block;

use nrf52840_hal as hal;

/// Exports traits that are usually needed when using this crate
pub mod prelude {
    pub use nrf52840_hal::prelude::*;
}

use hal::timer::{TimerExt, Timer};

pub mod led;
pub use led::{Led, Leds};
pub mod button;
pub use button::Button;
pub mod board;
pub use board::Board;

pub fn leds_welcome<T>(leds: &mut Leds, timer: &mut Timer<T>)
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

pub fn delay<T>(timer: &mut Timer<T>, cycles: u32)
    where
        T: TimerExt,
{
    timer.start(cycles);
    drop(block!(timer.wait()));
}
