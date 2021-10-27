#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32h7xx_hal::{prelude::*, spi, timer};

//use nb::block;

#[entry]
fn main() -> ! {
    let dp = stm32h7xx_hal::stm32::Peripherals::take().unwrap();

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // Constrain and Freeze clock
    let rcc = dp.RCC.constrain();
    let rcc = rcc.use_hse(8.mhz()).bypass_hse().sys_ck(400.mhz()).pll1_q_ck(200.mhz());
    let ccdr = rcc.freeze(pwrcfg, &dp.SYSCFG);

    // Acquire the GPIOC peripheral. This also enables the clock for
    // GPIOA in the RCC register.
    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    let sck = gpioa.pa5.into_alternate_af5();
    let miso = gpioa.pa6.into_alternate_af5();
    let mosi = gpiob.pb5.into_alternate_af5();

    // Initialise the SPI peripheral.
    let mut spi = dp.SPI1.spi(
        (sck, miso, mosi),
        spi::MODE_1,
        1.mhz(),
        ccdr.peripheral.SPI1,
        &ccdr.clocks,
    );

    // Write fixed data
    spi.write(&[0x11u8, 0x22, 0x33]).unwrap();

    // Configure the timer to trigger an update every second
    let mut timer = timer::Timer::tim1(dp.TIM1, ccdr.peripheral.TIM1, &ccdr.clocks);
    timer.start(1.hz());
    // Echo what is received on the SPI
    //let mut received = 0;
    loop {
        spi.write(&[0x11u8, 0x22, 0x33]).unwrap();
        nb::block!(timer.wait()).unwrap();
        //received = block!(spi.read()).unwrap();
    }
}
