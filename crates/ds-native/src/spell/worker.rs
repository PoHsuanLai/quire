//! The checker's worker thread: one per [`SpellConfig`](super::SpellConfig), fed by a channel,
//! answering on oneshot channels. Checking a paragraph and building suggestions take
//! milliseconds, loading a dictionary tens of them; none of it may run on the UI thread.

use super::books::Books;
use super::config::SpellConfig;
use ds::{Lang, Learned};
use std::sync::mpsc::{Sender, channel};
use tokio::sync::oneshot;

/// One piece of work.
pub(crate) enum Job {
    Check {
        langs: Vec<Lang>,
        words: Vec<String>,
        reply: oneshot::Sender<Vec<String>>,
    },
    Suggest {
        langs: Vec<Lang>,
        word: String,
        reply: oneshot::Sender<Vec<String>>,
    },
    Ignore(String),
    Learn {
        lang: Lang,
        word: String,
        reply: oneshot::Sender<Learned>,
    },
}

/// Start a worker over `config`'s dictionaries. It ends when the last sender is dropped.
pub(crate) fn start(config: SpellConfig) -> Sender<Job> {
    let (jobs, inbox) = channel::<Job>();
    // A worker that cannot start leaves every job unanswered: each surface then shows no marks
    // (its checks resolve to nothing) rather than the app failing.
    let _ = std::thread::Builder::new()
        .name("spellcheck".to_owned())
        .spawn(move || {
            let mut books = Books::new(config);
            for job in inbox {
                work(&mut books, job);
            }
        });
    jobs
}

fn work(books: &mut Books, job: Job) {
    match job {
        Job::Check {
            langs,
            words,
            reply,
        } => {
            if !reply.is_closed() {
                let _ = reply.send(books.check(&langs, words));
            }
        }
        Job::Suggest { langs, word, reply } => {
            if !reply.is_closed() {
                let _ = reply.send(books.suggest(&langs, &word));
            }
        }
        Job::Ignore(word) => books.ignore(word),
        Job::Learn { lang, word, reply } => {
            let _ = reply.send(books.learn(&lang, &word));
        }
    }
}
