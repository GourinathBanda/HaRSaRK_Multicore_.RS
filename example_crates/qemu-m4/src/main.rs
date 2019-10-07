#![no_std]
#![no_main]
#![feature(log_syntax)]

extern crate panic_halt;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::hprintln;

use stm32f4::stm32f407;
use stm32f4::stm32f407::interrupt;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use hartex_rust::process::*;
use hartex_rust::spawn;
use hartex_rust::types::*;
use hartex_rust::resource::{Resource,create};

#[entry]
fn main() -> ! {

    let app = create(7,14).unwrap();
    spawn!(thread1, 1, app, app, {
        app.acquire(|item| {
            hprintln!("{:?}", item);
        });
    });
    spawn!(thread2, 2, {
        // hprintln!("task 2  : {:?}", app);
    });
    spawn!(thread3, 3, app, app, {
        // hprintln!("task 3  : {:?}", app);
    });

    init(true);
    release(&14);
    start_kernel();

    loop {}
}
