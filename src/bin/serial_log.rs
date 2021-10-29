#![no_std]
#![no_main]

use panic_halt as _;

use nucleo_h743zi::logging;
use log::{info, warn, LevelFilter};
use stm32h7xx_hal::{block, serial, prelude::*, timer, device};
use cortex_m_log::{printer, printer::Printer};
use cortex_m::interrupt::{self, Mutex};
use core::{borrow::Borrow, cell::RefCell};

use cortex_m_rt::entry;

static SERIAL_REF: Mutex<RefCell<Option<serial::Serial<device::USART3>>>> = Mutex::new(RefCell::new(None));

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
    let rcc = rcc.sys_ck(400.mhz()).use_hse(8.mhz()).bypass_hse();
    let ccdr = rcc.freeze(pwrcfg, &dp.SYSCFG);

    // Acquire the GPIOD peripheral
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);

    // Initialize serial
    gpiod.pd8.into_alternate_af7();
    gpiod.pd9.into_alternate_af7();

    let serial = serial::Serial::usart3(
        dp.USART3,
        serial::config::Config::default().baudrate(115200.bps()),
        ccdr.peripheral.USART3,
        &ccdr.clocks
    ).unwrap();

    // Configure the timer to trigger an update every second
    let mut timer = timer::Timer::tim1(dp.TIM1, ccdr.peripheral.TIM1, &ccdr.clocks);
    timer.start(1.hz());

    let ser_printer = printer::generic::GenericPrinter::new(serial);

    let cortex_logger = cortex_m_log::log::Logger {
        level: LevelFilter::Info,
        inner: ser_printer
    };

    unsafe { 
        cortex_m_log::log::trick_init(&cortex_logger).unwrap();
    }
    // Configure the serial port as a logger
    //logging::serial::init(serial);
    info!("Serial logger example application");
    loop {
        info!("Hello, I'm a periodic printout");
        warn!("Hello, I'm a warning!");
        block!(timer.wait()).unwrap();
    }

}
