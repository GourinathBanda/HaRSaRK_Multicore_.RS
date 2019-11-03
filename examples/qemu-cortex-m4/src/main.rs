#![no_std]
#![no_main]

extern crate panic_halt;
extern crate stm32f4;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use hartex_rust::events;
use hartex_rust::util::generate_task_mask;
use hartex_rust::messages;
use hartex_rust::tasks::*;
use hartex_rust::resources;
use hartex_rust::semaphores;
use hartex_rust::types::*;
use hartex_rust::spawn;

struct app {
    sem2: SemaphoreId,
    msg1: Message<[u32; 2]>,
}

#[entry]
fn main() -> ! {
    let peripherals = resources::init_peripherals().unwrap();

    let app_inst = app {
        sem2: semaphores::new(generate_task_mask(&[2])).unwrap(),
        msg1: message::create(generate_task_mask(&[3]), generate_task_mask(&[3]), [9, 10]).unwrap(),
    };

    let e1 = event::new_FreeRunning(true, 1, EventTableType::Sec).unwrap();
    event::set_tasks(e1, generate_task_mask(&[1]));

    let e2 = event::new_FreeRunning(true, 2, EventTableType::Sec).unwrap();
    event::set_semaphore(e2, app_inst.sem2, generate_task_mask(&[1, 2]));

    let e3 = event::new_FreeRunning(false, 3, EventTableType::Sec).unwrap();
    event::set_message(e3, app_inst.msg1.get_id());

    let e4 = event::new_OnOff(true).unwrap();
    event::set_next_event(e4, e3);

    static mut stack1: [u32; 300] = [0; 300];
    static mut stack2: [u32; 300] = [0; 300];
    static mut stack3: [u32; 300] = [0; 300];

    spawn!(thread1, 1, stack1, params, app_inst, {
        hprintln!("TASK 1: Enter");
        if let Ok(x) = semaphores::test_and_reset(params.sem2) {
            if (x) {
                hprintln!("TASK 1: sem2 enabled");
            }
        }
        hprintln!("TASK 1: End");
    });
    spawn!(thread2, 2, stack2, params, app_inst, {
        hprintln!("TASK 2: Enter");
        if let Ok(x) = semaphores::test_and_reset(params.sem2) {
            hprintln!("TASK 2: sem2 enabled");
        }
        hprintln!("TASK 2: End");
    });
    spawn!(thread3, 3, stack3, params, app_inst, {
        hprintln!("TASK 3: Enter");
        if let Some(msg) = params.msg1.receive() {
            hprintln!("TASK 3: msg received : {:?}", msg);
        }
        hprintln!("TASK 3: End");
    });

    init(true);
    release(1);
    start_kernel(unsafe{&mut peripherals.access().unwrap().borrow_mut()}, 150_000);loop {}
}
