//! Board support crate for the makerdiary.com nRF52840-MDK

//pub use embedded_hal;
//use embedded_hal::digital::v2::OutputPin;

use crate::led::{Led, Leds};
use crate::button::Button;

use crate::hal::{
  prelude::*,
  gpio::{
    p0,
    Pin,
    Floating,
    Input,
    Output,
    PushPull,
    Level,
  },
  nrf52840_pac::{
    self as nrf52,
    CorePeripherals,
    Peripherals,
  },
  spim::{
    self,
    Frequency,
    MODE_0,
    Spim
  },
  uarte::{
    self,
    Uarte,
    Parity as UartParity,
    Baudrate as UartBaudrate,
  },
};


/// The nRF52 pins that are available on the nRF52840DK
pub struct Pins {
  pub p04: p0::P0_04<Input<Floating>>,
  pub p05: p0::P0_05<Input<Floating>>,
  pub p06: p0::P0_06<Input<Floating>>,
  pub p07: p0::P0_07<Input<Floating>>,
  pub p08: p0::P0_08<Input<Floating>>,
  pub p09: p0::P0_09<Input<Floating>>,
  pub p10: p0::P0_10<Input<Floating>>,
  pub p11: p0::P0_11<Input<Floating>>,
  pub p12: p0::P0_12<Input<Floating>>,
  pub p13: p0::P0_13<Input<Floating>>,
  pub p14: p0::P0_14<Input<Floating>>,
  pub p15: p0::P0_15<Input<Floating>>,
  pub p16: p0::P0_16<Input<Floating>>,
  pub p17: p0::P0_17<Input<Floating>>,
  pub p21: p0::P0_21<Input<Floating>>,

  pub p03: p0::P0_03<Input<Floating>>,
  pub p02: p0::P0_02<Input<Floating>>,
  pub p31: p0::P0_31<Input<Floating>>,
  pub p30: p0::P0_30<Input<Floating>>,
  pub p29: p0::P0_29<Input<Floating>>,
  pub p28: p0::P0_28<Input<Floating>>,
  pub p27: p0::P0_27<Input<Floating>>,
  pub p26: p0::P0_26<Input<Floating>>,
  pub p25: p0::P0_25<Input<Floating>>,
}

/// The Flash on the nRF52840-MDK board
pub struct Flash {
  pub spim: Spim<nrf52::SPIM2>,
  pub cs: Pin<Output<PushPull>>,
}

/// Provides access to all features of the nRF52840-MDK board
#[allow(non_snake_case)]
pub struct Board {
  /// The nRF52's pins which are not otherwise occupied on the nRF52840-MDK
  pub pins: Pins,

  /// The nRF52840-MDK UART which is wired to the DAPLink USB CDC port
  pub uart_daplink: Uarte<nrf52::UARTE0>,

  /// The nRF52840-MDK SPI which is wired to the SPI flash
  pub flash: Flash,

  /// The LEDs on the nRF52840-MDK board
  pub leds: Leds,

  /// The buttons on the nRF52840-MDK board
  pub button: Button,

  /// Core peripheral: Cache and branch predictor maintenance operations
  pub CBP: nrf52::CBP,

  /// Core peripheral: CPUID
  pub CPUID: nrf52::CPUID,

  /// Core peripheral: Debug Control Block
  pub DCB: nrf52::DCB,

  /// Core peripheral: Data Watchpoint and Trace unit
  pub DWT: nrf52::DWT,

  /// Core peripheral: Flash Patch and Breakpoint unit
  pub FPB: nrf52::FPB,

  /// Core peripheral: Floating Point Unit
  pub FPU: nrf52::FPU,

  /// Core peripheral: Instrumentation Trace Macrocell
  pub ITM: nrf52::ITM,

  /// Core peripheral: Memory Protection Unit
  pub MPU: nrf52::MPU,

  /// Core peripheral: Nested Vector Interrupt Controller
  pub NVIC: nrf52::NVIC,

  /// Core peripheral: System Control Block
  pub SCB: nrf52::SCB,

  /// Core peripheral: SysTick Timer
  pub SYST: nrf52::SYST,

  /// Core peripheral: Trace Port Interface Unit
  pub TPIU: nrf52::TPIU,

  /// nRF52 peripheral: FICR
  pub FICR: nrf52::FICR,

  /// nRF52 peripheral: UICR
  pub UICR: nrf52::UICR,

  /// nRF52 peripheral: ACL
  pub ACL: nrf52::ACL,

  /// nRF52 peripheral: POWER
  pub POWER: nrf52::POWER,

  /// nRF52 peripheral: CLOCK
  pub CLOCK: nrf52::CLOCK,

  /// nRF52 peripheral: RADIO
  pub RADIO: nrf52::RADIO,

  /// nRF52 peripheral: UART0
  pub UART0: nrf52::UART0,

  /// nRF52 peripheral: SPIM0
  pub SPIM0: nrf52::SPIM0,

  /// nRF52 peripheral: SPIS0
  pub SPIS0: nrf52::SPIS0,

  /// nRF52 peripheral: TWIM0
  pub TWIM0: nrf52::TWIM0,

  /// nRF52 peripheral: TWIS0
  pub TWIS0: nrf52::TWIS0,

  /// nRF52 peripheral: SPI0
  pub SPI0: nrf52::SPI0,

  /// nRF52 peripheral: TWI0
  pub TWI0: nrf52::TWI0,

  /// nRF52 peripheral: SPIM1
  pub SPIM1: nrf52::SPIM1,

  /// nRF52 peripheral: SPIS1
  pub SPIS1: nrf52::SPIS1,

  /// nRF52 peripheral: TWIS1
  pub TWIS1: nrf52::TWIS1,

  /// nRF52 peripheral: SPI1
  pub SPI1: nrf52::SPI1,

  /// nRF52 peripheral: TWI1
  pub TWI1: nrf52::TWI1,

  /// nRF52 peripheral: NFCT
  pub NFCT: nrf52::NFCT,

  /// nRF52 peripheral: GPIOTE
  pub GPIOTE: nrf52::GPIOTE,

  /// nRF52 peripheral: SAADC
  pub SAADC: nrf52::SAADC,

  /// nRF52 peripheral: TIMER0
  pub TIMER0: nrf52::TIMER0,

  /// nRF52 peripheral: TIMER1
  pub TIMER1: nrf52::TIMER1,

  /// nRF52 peripheral: TIMER2
  pub TIMER2: nrf52::TIMER2,

  /// nRF52 peripheral: RTC0
  pub RTC0: nrf52::RTC0,

  /// nRF52 peripheral: TEMP
  pub TEMP: nrf52::TEMP,

  /// nRF52 peripheral: RNG
  pub RNG: nrf52::RNG,

  /// nRF52 peripheral: ECB
  pub ECB: nrf52::ECB,

  /// nRF52 peripheral: CCM
  pub CCM: nrf52::CCM,

  /// nRF52 peripheral: AAR
  pub AAR: nrf52::AAR,

  /// nRF52 peripheral: WDT
  pub WDT: nrf52::WDT,

  /// nRF52 peripheral: RTC1
  pub RTC1: nrf52::RTC1,

  /// nRF52 peripheral: QDEC
  pub QDEC: nrf52::QDEC,

  /// nRF52 peripheral: COMP
  pub COMP: nrf52::COMP,

  /// nRF52 peripheral: LPCOMP
  pub LPCOMP: nrf52::LPCOMP,

  /// nRF52 peripheral: SWI0
  pub SWI0: nrf52::SWI0,

  /// nRF52 peripheral: EGU0
  pub EGU0: nrf52::EGU0,

  /// nRF52 peripheral: SWI1
  pub SWI1: nrf52::SWI1,

  /// nRF52 peripheral: EGU1
  pub EGU1: nrf52::EGU1,

  /// nRF52 peripheral: SWI2
  pub SWI2: nrf52::SWI2,

  /// nRF52 peripheral: EGU2
  pub EGU2: nrf52::EGU2,

  /// nRF52 peripheral: SWI3
  pub SWI3: nrf52::SWI3,

  /// nRF52 peripheral: EGU3
  pub EGU3: nrf52::EGU3,

  /// nRF52 peripheral: SWI4
  pub SWI4: nrf52::SWI4,

  /// nRF52 peripheral: EGU4
  pub EGU4: nrf52::EGU4,

  /// nRF52 peripheral: SWI5
  pub SWI5: nrf52::SWI5,

  /// nRF52 peripheral: EGU5
  pub EGU5: nrf52::EGU5,

  /// nRF52 peripheral: TIMER3
  pub TIMER3: nrf52::TIMER3,

  /// nRF52 peripheral: TIMER4
  pub TIMER4: nrf52::TIMER4,

  /// nRF52 peripheral: PWM0
  pub PWM0: nrf52::PWM0,

  /// nRF52 peripheral: PDM
  pub PDM: nrf52::PDM,

  /// nRF52 peripheral: NVMC
  pub NVMC: nrf52::NVMC,

  /// nRF52 peripheral: PPI
  pub PPI: nrf52::PPI,

  /// nRF52 peripheral: MWU
  pub MWU: nrf52::MWU,

  /// nRF52 peripheral: PWM1
  pub PWM1: nrf52::PWM1,

  /// nRF52 peripheral: PWM2
  pub PWM2: nrf52::PWM2,

  /// nRF52 peripheral: RTC2
  pub RTC2: nrf52::RTC2,

  /// nRF52 peripheral: I2S
  pub I2S: nrf52::I2S,
}

impl Board {
  /// Take the peripherals safely
  ///
  /// This method will return an instance of `nRF52840MDK` the first time it is
  /// called. It will return only `None` on subsequent calls.
  pub fn take() -> Option<Self> {
    Some(Self::new(
      CorePeripherals::take()?,
      Peripherals::take()?,
    ))
  }

  /// Steal the peripherals
  ///
  /// This method produces an instance of `nRF52840MDK`, regardless of whether
  /// another instance was create previously.
  ///
  /// # Safety
  ///
  /// This method can be used to create multiple instances of `nRF52840MDK`. Those
  /// instances can interfere with each other, causing all kinds of unexpected
  /// behavior and circumventing safety guarantees in many ways.
  ///
  /// Always use `nRF52840MDK::take`, unless you really know what you're doing.
//  pub unsafe fn steal() -> Self {
//    Self::new(
//      CorePeripherals::steal(),
//      Peripherals::steal(),
//    )
//  }

  fn new(cp: CorePeripherals, p: Peripherals) -> Self {
    let pins0 = p.P0.split();
    let pins1 = p.P1.split();

    // The nRF52840-MDK features an USB CDC port.
    // It features HWFC but does not have to use it.
    // It can transmit a flexible baudrate of up to 1Mbps.
    let uart_daplink_pins = uarte::Pins {
      txd: pins0.p0_20.into_push_pull_output(Level::High).degrade(),
      rxd: pins0.p0_19.into_floating_input().degrade(),
      cts: None,
      rts: None,
    };
    let uart_daplink = p.UARTE0.constrain(uart_daplink_pins,
                                          UartParity::EXCLUDED,
                                          UartBaudrate::BAUD115200);

    // The nRF52840-MDK has an 64MB SPI flash on board which can be interfaced through SPI or Quad SPI.
    // As for now, only the normal SPI mode is available, so we are using this for the interface.
    let flash_spim_pins = spim::Pins {
      sck : pins1.p1_03.into_push_pull_output(Level::Low).degrade(),
      mosi: Some(pins1.p1_05.into_push_pull_output(Level::Low).degrade()),
      miso: Some(pins1.p1_04.into_floating_input().degrade()),
    };
    let flash = Flash {
      spim: p.SPIM2.constrain(flash_spim_pins, Frequency::K500, MODE_0, 0),
      cs: pins1.p1_06.into_push_pull_output(Level::High).degrade(),
    };

    let pins = Pins {
      p04: pins0.p0_04,
      p05: pins0.p0_05,
      p06: pins0.p0_06,
      p07: pins0.p0_07,
      p08: pins0.p0_08,
      p09: pins0.p0_09,
      p10: pins0.p0_10,
      p11: pins0.p0_11,
      p12: pins0.p0_12,
      p13: pins0.p0_13,
      p14: pins0.p0_14,
      p15: pins0.p0_15,
      p16: pins0.p0_16,
      p17: pins0.p0_17,
      p21: pins0.p0_21,

      p03: pins0.p0_03,
      p02: pins0.p0_02,
      p31: pins0.p0_31,
      p30: pins0.p0_30,
      p29: pins0.p0_29,
      p28: pins0.p0_28,
      p27: pins0.p0_27,
      p26: pins0.p0_26,
      p25: pins0.p0_25,
    };

    let leds = Leds {
      red: Led::new(pins0.p0_23.degrade()),
      green: Led::new(pins0.p0_22.degrade()),
      blue: Led::new(pins0.p0_24.degrade()),
    };

    let button = Button::new(pins1.p1_00.degrade());

    Board {
      uart_daplink,
      flash,
      pins,
      leds,
      button,

      // Core peripherals
      CBP  : cp.CBP,
      CPUID: cp.CPUID,
      DCB  : cp.DCB,
      DWT  : cp.DWT,
      FPB  : cp.FPB,
      FPU  : cp.FPU,
      ITM  : cp.ITM,
      MPU  : cp.MPU,
      NVIC : cp.NVIC,
      SCB  : cp.SCB,
      SYST : cp.SYST,
      TPIU : cp.TPIU,

      // nRF52 peripherals
      FICR  : p.FICR,
      UICR  : p.UICR,
      ACL   : p.ACL,
      POWER : p.POWER,
      CLOCK : p.CLOCK,
      RADIO : p.RADIO,

      UART0 : p.UART0,
      SPIM0 : p.SPIM0,
      SPIS0 : p.SPIS0,
      TWIM0 : p.TWIM0,
      TWIS0 : p.TWIS0,
      SPI0  : p.SPI0,
      TWI0  : p.TWI0,
      SPIM1 : p.SPIM1,
      SPIS1 : p.SPIS1,
      TWIS1 : p.TWIS1,
      SPI1  : p.SPI1,
      TWI1  : p.TWI1,
      NFCT  : p.NFCT,
      GPIOTE: p.GPIOTE,
      SAADC : p.SAADC,
      TIMER0: p.TIMER0,
      TIMER1: p.TIMER1,
      TIMER2: p.TIMER2,
      RTC0  : p.RTC0,
      TEMP  : p.TEMP,
      RNG   : p.RNG,
      ECB   : p.ECB,
      CCM   : p.CCM,
      AAR   : p.AAR,
      WDT   : p.WDT,
      RTC1  : p.RTC1,
      QDEC  : p.QDEC,
      COMP  : p.COMP,
      LPCOMP: p.LPCOMP,
      SWI0  : p.SWI0,
      EGU0  : p.EGU0,
      SWI1  : p.SWI1,
      EGU1  : p.EGU1,
      SWI2  : p.SWI2,
      EGU2  : p.EGU2,
      SWI3  : p.SWI3,
      EGU3  : p.EGU3,
      SWI4  : p.SWI4,
      EGU4  : p.EGU4,
      SWI5  : p.SWI5,
      EGU5  : p.EGU5,
      TIMER3: p.TIMER3,
      TIMER4: p.TIMER4,
      PWM0  : p.PWM0,
      PDM   : p.PDM,
      NVMC  : p.NVMC,
      PPI   : p.PPI,
      MWU   : p.MWU,
      PWM1  : p.PWM1,
      PWM2  : p.PWM2,
      RTC2  : p.RTC2,
      I2S   : p.I2S,
    }
  }
}
