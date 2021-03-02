//! Message primitive
//!

use core::cell::RefCell;

use crate::system::scheduler::BooleanVector;
use crate::system::scheduler::Scheduler;
use crate::system::semaphore::Semaphore;
use crate::tasks::get_curr_tid;
use crate::utils::arch::{critical_section, Mutex};

#[cfg(feature = "system_logger")]
use {crate::kernel::logging, crate::system::system_logger::LogEventType};

/// Holds metadata corresponding to a single message object.
pub struct Message<T: Sized + Clone> {
    value: RefCell<T>,
    pub receivers: BooleanVector,
    semaphore: Semaphore,
}

impl<T: Sized + Clone> Message<T> {
    /// Create and initialize new message object
    pub const fn new(
        task_manager: &'static Mutex<RefCell<Scheduler>>,
        tasks_mask: BooleanVector,
        receivers_mask: BooleanVector,
        value: T,
    ) -> Self {
        Self {
            value: RefCell::new(value),
            receivers: receivers_mask,
            semaphore: Semaphore::new(task_manager, tasks_mask),
        }
    }

    /// Broadcast the message to all reciever tasks
    pub fn broadcast(&'static self, msg: Option<T>) {
        critical_section(|_| {
            if let Some(msg) = msg {
                self.value.replace(msg);
            }
            self.semaphore.signal_and_release(self.receivers);
            #[cfg(feature = "system_logger")]
            {
                if logging::get_message_broadcast() {
                    logging::report(LogEventType::MessageBroadcast(self.receivers));
                }
            }
        })
    }

    /// Get a copy of the messsage on recieving a message
    pub fn receive(&'static self) -> Option<T> {
        critical_section(|_| match self.semaphore.test_and_reset() {
            Ok(res) if res == true => {
                #[cfg(feature = "system_logger")]
                {
                    if logging::get_message_recieve() {
                        // TODO: Fix
                        // logging::report(LogEventType::MessageRecieve(get_curr_tid() as u32));
                    }
                }
                Some(self.value.borrow().clone())
            }
            _ => None,
        })
    }
}

unsafe impl<T: Sized + Clone> Sync for Message<T> {}
