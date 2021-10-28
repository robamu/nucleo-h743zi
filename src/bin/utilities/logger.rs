use lazy_static::lazy_static;
use log::LevelFilter;

pub use cortex_m_log::log::Logger;

use cortex_m_log::{
    destination::Itm as ItmDest,
    printer::itm::InterruptSync,
    modes::InterruptFree,
    printer::itm::ItmSync
};

lazy_static! {
    static ref ITM_LOGGER: Logger<ItmSync<InterruptFree>> = Logger {
        level: LevelFilter::Info,
        inner: unsafe {
            InterruptSync::new(
                ItmDest::new(cortex_m::Peripherals::steal().ITM)
            )
        },
    };
}

pub fn init_itm_log() {
    cortex_m_log::log::init(&ITM_LOGGER).unwrap();
}
