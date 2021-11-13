#![no_std]
#![no_main]

mod consts;

use base as _;

#[rtic::app(device = hal::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    #[cfg(feature = "heap")]
    use {crate::consts::HEAP_SIZE, cortex_m_rt};
    use {
        base::watchdog::{groomer_task, init_watchdogs},
        hal::{
            gpio::{Level, Output, Pin, PushPull},
            wdt::{handles::HdlN, WatchdogHandle},
        },
        nrf52840_hal as hal,
        rtt_target::{rprintln, rtt_init_print},
        systick_monotonic::*,
    };

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led0: Pin<Output<PushPull>>,
        led1: Pin<Output<PushPull>>,
        led2: Pin<Output<PushPull>>,
        led3: Pin<Output<PushPull>>,
        wd0: WatchdogHandle<HdlN>,
        wd1: WatchdogHandle<HdlN>,
        wd2: WatchdogHandle<HdlN>,
        wd3: WatchdogHandle<HdlN>,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("[Init]");
        let _clocks = hal::clocks::Clocks::new(cx.device.CLOCK).enable_ext_hfosc();

        let p0 = hal::gpio::p0::Parts::new(cx.device.P0);
        let led0 = p0.p0_13.into_push_pull_output(Level::High).degrade();
        let led1 = p0.p0_14.into_push_pull_output(Level::High).degrade();
        let led2 = p0.p0_15.into_push_pull_output(Level::High).degrade();
        let led3 = p0.p0_16.into_push_pull_output(Level::High).degrade();

        let (wd0, wd1, wd2, wd3) = init_watchdogs(cx.device.WDT);

        // Enable the monotonic timer (CYCCNT)
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();
        rprintln!("Starting!");

        if cx.device.POWER.resetreas.read().dog().is_detected() {
            cx.device.POWER.resetreas.modify(|_r, w| {
                // Clear the watchdog reset reason bit
                w.dog().set_bit()
            });
            rprintln!("Restarted by the dog!");
        } else {
            rprintln!("Not restarted by the dog!");
        }

        // Test the allocator if heap is enabled.
        #[cfg(feature = "heap")]
        {
            rprintln!("Testing heap allocation!");
            unsafe {
                base::ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE);
            }
            extern crate alloc;
            use alloc::vec;
            let test = vec![0..4];
            rprintln!("{:?}", test);
        }

        let systick = cx.core.SYST;
        let mono = Systick::new(systick, 12_000_000);
        tick::spawn_after(1.secs()).unwrap();

        (
            Shared {},
            Local {
                led0,
                led1,
                led2,
                led3,
                wd0: wd0.degrade(),
                wd1: wd1.degrade(),
                wd2: wd2.degrade(),
                wd3: wd3.degrade(),
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task()]
    fn tick(_cx: tick::Context) {
        groomer0::spawn().unwrap();
        groomer1::spawn().unwrap();
        groomer2::spawn().unwrap();
        groomer3::spawn().unwrap();
        tick::spawn_after(1.secs()).unwrap();
    }

    #[task(local = [led0, wd0])]
    fn groomer0(mut cx: groomer0::Context) {
        #[cfg(feature = "good-dog")]
        groomer_task(&mut cx.local.led0, &mut cx.local.wd0);
    }

    #[task(local = [led1, wd1])]
    fn groomer1(mut cx: groomer1::Context) {
        #[cfg(feature = "good-dog")]
        groomer_task(&mut cx.local.led1, &mut cx.local.wd1);
    }

    #[task(local = [led2, wd2])]
    fn groomer2(mut cx: groomer2::Context) {
        #[cfg(feature = "good-dog")]
        groomer_task(&mut cx.local.led2, &mut cx.local.wd2);
    }

    #[task(local = [led3, wd3])]
    fn groomer3(mut cx: groomer3::Context) {
        #[cfg(feature = "good-dog")]
        groomer_task(&mut cx.local.led3, &mut cx.local.wd3);
    }
}
