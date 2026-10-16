//! Running a call to a peer under a [`Deadline`].

use crate::Deadline;
use std::future::{Future, poll_fn};
use std::pin::{Pin, pin};
use std::task::Poll;
use std::time::Duration;

/// A peer that did not answer inside its deadline. The words are for a person: "The package
/// manager is not answering."
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{peer} is not answering")]
pub struct NotAnswering {
    peer: &'static str,
    after: Duration,
}

impl NotAnswering {
    /// Who did not answer, as a screen names it ("The package manager").
    pub fn peer(&self) -> &'static str {
        self.peer
    }

    /// How long it was given.
    pub fn after(&self) -> Duration {
        self.after
    }
}

/// Run `work` for at most `deadline`. `peer` is the plain name a screen would use for whoever is
/// being waited on ("The package manager", "Accounts"); it starts with a capital and names no
/// protocol. The call is dropped, and so cancelled at its next await, when the deadline passes.
pub async fn within<T>(
    deadline: Deadline,
    peer: &'static str,
    work: impl Future<Output = T>,
) -> Result<T, NotAnswering> {
    let mut work = pin!(work);
    let mut timer = futures_timer::Delay::new(deadline.duration());
    poll_fn(move |cx| {
        if let Poll::Ready(done) = work.as_mut().poll(cx) {
            return Poll::Ready(Ok(done));
        }
        match Pin::new(&mut timer).poll(cx) {
            Poll::Ready(()) => Poll::Ready(Err(NotAnswering {
                peer,
                after: deadline.duration(),
            })),
            Poll::Pending => Poll::Pending,
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::{Deadline, NotAnswering, within};
    use std::future::pending;
    use std::time::Duration;

    const SHORT: Deadline = Deadline::of(Duration::from_millis(30));

    #[tokio::test]
    async fn a_call_that_answers_in_time_gives_its_answer() {
        assert_eq!(within(SHORT, "Accounts", async { 7 }).await, Ok(7));
    }

    #[tokio::test]
    async fn a_call_that_never_answers_ends_as_not_answering() {
        let got = within(SHORT, "Accounts", pending::<()>()).await;
        let Err(error) = got else {
            panic!("a pending call cannot answer");
        };
        assert_eq!(error.peer(), "Accounts");
        assert_eq!(error.after(), SHORT.duration());
        assert_eq!(error.to_string(), "Accounts is not answering");
    }

    #[tokio::test]
    async fn a_late_answer_is_not_waited_for() {
        let slow = async {
            futures_timer::Delay::new(Duration::from_secs(30)).await;
        };
        let started = std::time::Instant::now();
        let got: Result<(), NotAnswering> = within(SHORT, "Accounts", slow).await;
        assert!(got.is_err());
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
