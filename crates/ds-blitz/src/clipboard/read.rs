//! Reading the system clipboard without letting its owner wedge the app.
//!
//! Another program owns what is on the clipboard and is asked for it at the moment of the paste.
//! One that keeps the pipe open and never writes would freeze a read made on the UI thread for
//! good, and one that sends gigabytes would fill memory. So a read runs on a worker thread; the
//! UI waits for it at most [`READ_WAIT`], takes at most [`MAX_PASTE_BYTES`], and starts no second
//! read while an earlier one is still stuck, so a stuck owner costs one idle thread and no more.

use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

/// How long a paste waits for the clipboard's owner. A paste is a keystroke the person is
/// watching, so this is short; an owner that misses it is read as having nothing to give.
const READ_WAIT: Duration = Duration::from_millis(500);

/// The most text one paste takes: 16 MiB, far more than anyone types and far less than memory.
const MAX_PASTE_BYTES: usize = 16 * 1024 * 1024;

/// Runs clipboard reads off the calling thread, one at a time, under a time and a size limit.
#[derive(Debug)]
pub(super) struct Reader {
    /// One strong reference here, one more in each running read: more than one means a read is
    /// still out.
    running: Arc<()>,
    wait: Duration,
    cap: usize,
}

impl Default for Reader {
    fn default() -> Reader {
        Reader::with(READ_WAIT, MAX_PASTE_BYTES)
    }
}

impl Reader {
    fn with(wait: Duration, cap: usize) -> Reader {
        Reader {
            running: Arc::new(()),
            wait,
            cap,
        }
    }

    /// What `read` returns, run on a worker: `None` when it returns nothing, takes longer than the
    /// wait, returns more than the cap, or an earlier read has not ended.
    pub(super) fn text(
        &self,
        read: impl FnOnce() -> Option<String> + Send + 'static,
    ) -> Option<String> {
        if Arc::strong_count(&self.running) > 1 {
            return None;
        }
        let claim = Arc::clone(&self.running);
        let (send, receive) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("quire-clipboard-read".to_owned())
            .spawn(move || {
                let text = read();
                // Released before the answer is sent, so the next paste never finds it busy.
                drop(claim);
                let _ = send.send(text);
            })
            .ok()?;
        receive
            .recv_timeout(self.wait)
            .ok()
            .flatten()
            .filter(|text| text.len() <= self.cap)
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn a_read_that_never_ends_is_cut_and_holds_off_the_next_until_it_ends() {
        let reader = Reader::with(Duration::from_millis(40), 1024);
        let (release, hold) = mpsc::channel::<()>();
        let stuck = reader.text(move || {
            let _ = hold.recv();
            Some("late".to_owned())
        });
        assert_eq!(stuck, None);
        assert_eq!(
            reader.text(|| Some("second".to_owned())),
            None,
            "one stuck read at a time"
        );
        drop(release);
        let mut served = None;
        for _ in 0..200 {
            served = reader.text(|| Some("third".to_owned()));
            if served.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(served.as_deref(), Some("third"));
    }

    #[test]
    fn text_over_the_cap_is_refused_and_text_at_it_is_taken() {
        let reader = Reader::with(Duration::from_secs(5), 8);
        assert_eq!(
            reader.text(|| Some("12345678".to_owned())).as_deref(),
            Some("12345678")
        );
        assert_eq!(reader.text(|| Some("123456789".to_owned())), None);
    }

    #[test]
    fn a_read_that_panics_is_no_answer_and_does_not_hold_off_the_next() {
        let reader = Reader::with(Duration::from_secs(5), 8);
        assert_eq!(reader.text(|| panic!("an owner that cannot be read")), None);
        // The panicking thread releases its claim as it unwinds, a moment after the answer.
        let mut served = None;
        for _ in 0..200 {
            served = reader.text(|| Some("ok".to_owned()));
            if served.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(served.as_deref(), Some("ok"));
    }
}
