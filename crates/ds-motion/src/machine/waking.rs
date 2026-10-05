//! The one timer a machine runs: asleep until the state's `wake()`, replaced whenever that moves.

use super::handle::MachineRef;
use dioxus::core::Task;
use dioxus::prelude::*;
use ds_core::machine::Machine;
use ds_core::time::clock::sleep;
use ds_core::time::stamp::Stamp;
use ds_style::task::spawn_in;
use std::time::Duration;

/// The sleep that is running: the time it was set for and its task. None at rest.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Waking(Option<Sleeping>);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Sleeping {
    at: Stamp,
    task: Task,
}

impl<M: Machine> MachineRef<M> {
    /// Sleep until the state's next wake, replacing an earlier sleep for another time; none when
    /// the machine is at rest. A sleep already set for that time is kept, so following a state
    /// the machine itself just stepped starts nothing new.
    pub(super) fn rewake(self) {
        let want = self
            .held
            .state
            .try_peek()
            .ok()
            .and_then(|state| state.wake());
        let mut slot = self.waking;
        let Ok(Waking(held)) = slot.try_peek().map(|slot| *slot) else {
            return;
        };
        if held.map(|sleeping| sleeping.at) == want {
            return;
        }
        if let Some(sleeping) = held {
            sleeping.task.cancel();
        }
        let next = want.map(|at| Sleeping {
            at,
            task: self.sleep_until(at),
        });
        let _ = slot.try_write().map(|mut slot| *slot = Waking(next));
    }

    /// A task of the hook's own scope that wakes the machine at `at`, whichever component's
    /// handler asked for the sleep.
    fn sleep_until(self, at: Stamp) -> Task {
        let wait = Duration::from_millis(at.since(self.now()));
        spawn_in(self.scope, async move {
            sleep(wait).await;
            let mut slot = self.waking;
            if slot
                .try_write()
                .map(|mut slot| *slot = Waking(None))
                .is_ok()
            {
                self.elapsed();
            }
        })
    }
}
