//! Toggle an externally connected LED
//!
//! Tie an LED to the D2 pin with an appropriate resistor and then run this script to blinky the LED
//! with an interval of one second

#![no_std]
#![no_main]

use panic_halt as _;

use stm32h7xx_hal::{
    prelude::*,
    timer::Timer,
    block
};

use cortex_m_rt::entry;

use embedded_hal::digital::v2::OutputPin;

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = stm32h7xx_hal::stm32::Peripherals::take().unwrap();

    // Take ownership over the RCC devices and convert them into the corresponding HAL structs
    let rcc = dp.RCC.constrain();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // Freeze the configuration of all the clocks in the system and
    // retrieve the Core Clock Distribution and Reset (CCDR) object
    let rcc = rcc.use_hse(8.mhz()).bypass_hse();
    let ccdr = rcc.freeze(pwrcfg, &dp.SYSCFG);

    // Acquire the GPIOB peripheral
    let gpiof = dp.GPIOF.split(ccdr.peripheral.GPIOF);

    // Configure the D2 pin (GPIO F pin 15) as a push-pull output.
    let mut d2_pin = gpiof.pf15.into_push_pull_output();

    // Configure the timer to trigger an update every second
    let mut timer = Timer::tim1(dp.TIM1, ccdr.peripheral.TIM1, &ccdr.clocks);
    timer.start(1.hz());

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        d2_pin.set_high().unwrap();
        block!(timer.wait()).unwrap();
        d2_pin.set_low().unwrap();
        block!(timer.wait()).unwrap();
    }
}
