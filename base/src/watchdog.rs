use {
    embedded_hal::digital::v2::OutputPin,
    hal::{
        gpio::{Output, Pin, PushPull},
        pac::WDT,
        wdt::{
            count,
            handles::{Hdl0, Hdl1, Hdl2, Hdl3, HdlN},
            Parts, Watchdog, WatchdogHandle,
        },
    },
    nrf52840_hal as hal,
    rtt_target::rprintln,
};

pub fn groomer_task(led: &mut Pin<Output<PushPull>>, watchdog: &mut WatchdogHandle<HdlN>) {
    if !watchdog.is_pet() {
        watchdog.pet();
        led.set_low().ok();
    } else {
        led.set_high().ok();
    }
}

pub fn init_watchdogs(
    wdt: WDT,
) -> (
    WatchdogHandle<Hdl0>,
    WatchdogHandle<Hdl1>,
    WatchdogHandle<Hdl2>,
    WatchdogHandle<Hdl3>,
) {
    match Watchdog::try_new(wdt) {
        Ok(mut watchdog) => {
            // Set the watchdog to timeout after 5 seconds (in 32.768kHz ticks)
            watchdog.set_lfosc_ticks(5 * 32768);

            // Activate the watchdog with four handles
            let Parts {
                watchdog: _watchdog,
                handles,
            } = watchdog.activate::<count::Four>();

            return handles;
        }
        Err(wdt) => match Watchdog::try_recover::<count::Four>(wdt) {
            Ok(Parts { mut handles, .. }) => {
                rprintln!("Oops, watchdog already active, but recovering!");

                // Pet all the dogs quickly to reset to default timeout
                handles.0.pet();
                handles.1.pet();
                handles.2.pet();
                handles.3.pet();

                return handles;
            }
            Err(_wdt) => {
                rprintln!("Oops, watchdog already active, resetting!");
                panic!();
            }
        },
    };
}
