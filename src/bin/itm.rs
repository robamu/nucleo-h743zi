//! Shows the ITM functionality to get logger output
//! `itmdump` is used to display the outoput.
//! The README contains information on additional tools required.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m::iprintln;
use panic_halt as _;

use stm32h7xx_hal::{block, prelude::*, timer};

#[cfg(feature="enable-itm")]
use stm32h7xx_hal::{rcc, stm32};

#[cfg(feature="enable-itm")]
/// Enables ITM
unsafe fn enable_itm(
    dbgmcu: &stm32::DBGMCU,
    dcb: &mut cortex_m::peripheral::DCB,
    itm: &mut cortex_m::peripheral::ITM,
    clocks: &rcc::CoreClocks,
    swo_frequency: u32,
) {
    // ARMv7-M DEMCR: Set TRCENA. Enables DWT and ITM units
    //unsafe { *(0xE000_EDFC as *mut u32) |= 1 << 24 };
    dcb.enable_trace();

    // Ensure debug blocks are clocked before interacting with them
    dbgmcu.cr.modify(|_, w| {
        w.d1dbgcken()
            .set_bit()
            .d3dbgcken()
            .set_bit()
            .traceclken()
            .set_bit()
            .dbgsleep_d1()
            .set_bit()
    });

    // SWO: Unlock
    *(0x5c00_3fb0 as *mut u32) = 0xC5ACCE55;
    // SWTF: Unlock
    *(0x5c00_4fb0 as *mut u32) = 0xC5ACCE55;

    // SWO CODR Register: Set SWO speed
    *(0x5c00_3010 as *mut _) = clocks.c_ck().0 / swo_frequency - 1;

    // SWO SPPR Register:
    // 1 = Manchester
    // 2 = NRZ
    *(0x5c00_30f0 as *mut _) = 2;

    // SWTF Trace Funnel: Enable for CM7
    *(0x5c00_4000 as *mut u32) |= 1;

    // ITM: Unlock
    itm.lar.write(0xC5ACCE55);
    // ITM Trace Enable Register: Enable lower 8 stimulus ports
    itm.ter[0].write(1);
    // ITM Trace Control Register: Enable ITM
    itm.tcr.write(
        (0b000001 << 16) | // TraceBusID
        (1 << 3) | // enable SWO output
        (1 << 0), // enable the ITM
    );
}

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = stm32h7xx_hal::stm32::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    // Take ownership over the RCC devices and convert them into the corresponding HAL structs
    let rcc = dp.RCC.constrain();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // Freeze the configuration of all the clocks in the system and
    // retrieve the Core Clock Distribution and Reset (CCDR) object
    // Configure PLL1 R for TRACEC as well
    let rcc = rcc.sys_ck(400.mhz()).use_hse(8.mhz()).bypass_hse().pll1_r_ck(400.mhz());
    let ccdr = rcc.freeze(pwrcfg, &dp.SYSCFG);

    #[cfg(feature = "enable-itm")]
    unsafe {
        enable_itm(&dp.DBGMCU, &mut cp.DCB, &mut cp.ITM, &ccdr.clocks, 2000000);
    }

    let stim = &mut cp.ITM.stim[0];
    // Configure the timer to trigger an update every second
    let mut timer = timer::Timer::tim1(dp.TIM1, ccdr.peripheral.TIM1, &ccdr.clocks);
    timer.start(1.hz());

    iprintln!(stim, "Hello World");
    let mut counter = 0;

    loop {
        iprintln!(stim, "{}: This is a very useful logger", counter);
        counter += 1;
        // Echo what is received on the serial link.
        block!(timer.wait()).unwrap();
    }
}
