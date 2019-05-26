#![no_std]
#![no_main]

use panic_halt as _;

use stm32h7xx_hal::{
    stm32,
    prelude::*,
    serial::{self, Serial, Error},
};

use cortex_m_rt::entry;

use embedded_hal::digital::v2::OutputPin;

static mut MEM: f64 = 0.0;

fn spi2_setup(spi2: &stm32::SPI2) {
    // spi2.cfg1.modify(|_, w| {
    //     w.mbr().bits(0)  // clk/2
    //      .dsize().bits(16 - 1)
    //      .fthvl().one_frame()
    // });
    // spi2.cfg2.modify(|_, w| unsafe {
    //     w.afcntr().set_bit()
    //      .ssom().set_bit()  // ss deassert between frames during midi
    //      .ssoe().set_bit()  // ss output enable
    //      .ssiop().clear_bit()  // ss active low
    //      .ssm().clear_bit()  // PAD counts
    //      .cpol().clear_bit()
    //      .cpha().clear_bit()
    //      .lsbfrst().clear_bit()
    //      .master().set_bit()
    //      .sp().bits(0)  // motorola
    //      .comm().bits(0b01)  // simplex transmitter
    //      .ioswp().clear_bit()
    //      .midi().bits(0)  // master inter data idle
    //      .mssi().bits(0)  // master SS idle
    // });
    // spi2.cr2.modify(|_, w| w.tsize().bits(0));
    spi2.cr1.write(|w| w.spe().enabled());
    // spi2.cr1.modify(|_, w| w.cstart().started());
}

#[entry]
fn main() -> ! {
    let dp = stm32h7xx_hal::stm32::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb4);
    let mut gpioi = dp.GPIOI.split(&mut rcc.ahb4);

    let mut ld2 = gpiob.pb7
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut ld3 = gpiob.pb14
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let sck = gpiob.pb13.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let miso = gpioi.pi2.into_af5(&mut gpioi.moder, &mut gpioi.afrl);
    let mosi = gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);

    rcc.apb1.lenr().modify(|_, w| w.spi2en().set_bit());
    let spi2 = dp.SPI2;
    spi2_setup(&spi2);

    let mut count: u16 = 0;
    loop {

        if count % 2 == 0 {
            embedded_hal::digital::v2::OutputPin::set_low(&mut ld2).unwrap();
            embedded_hal::digital::v2::OutputPin::set_high(&mut ld3).unwrap();
        } else {
            embedded_hal::digital::v2::OutputPin::set_high(&mut ld2).unwrap();
            embedded_hal::digital::v2::OutputPin::set_low(&mut ld3).unwrap();
        }

        let d = count;

        let txdr = &spi2.txdr as *const _ as *mut u16;
        unsafe { core::ptr::write_volatile(txdr, d) };
        spi2.cr1.modify(|_, w| w.cstart().started());

        count += 1;

        delay();

    }
}

fn delay() {
    let mut count = 0;
    loop {
        let x = count as f64 * 1.00001;

        unsafe { core::ptr::write_volatile(&mut MEM, x); }

        if x > 5e3 {
            break;
        }
        count += 1;
    }
}
