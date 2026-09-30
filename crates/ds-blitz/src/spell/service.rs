//! [`NativeSpell`]: `ds::SpellService` on Blitz, over the worker.

use super::choose::locale_lang;
use super::config::SpellConfig;
use super::paragraphs::paragraphs;
use super::worker::{Job, start};
use dioxus::prelude::MountedData;
use dioxus_native_dom::NodeHandle;
use ds::{Lang, Learned, Paragraph, Probe, SpellFuture, SpellService};
use std::sync::mpsc::Sender;
use std::sync::{LazyLock, Mutex, PoisonError};
use tokio::sync::oneshot;

/// The process's one worker over the system's dictionaries, shared by every surface, so each
/// dictionary loads once and "Ignore Spelling" holds for the whole session.
static SYSTEM: LazyLock<Mutex<Sender<Job>>> =
    LazyLock::new(|| Mutex::new(start(SpellConfig::system())));

/// A spellchecker on a worker thread.
#[derive(Debug, Clone)]
pub struct NativeSpell {
    jobs: Sender<Job>,
    languages: Vec<Lang>,
}

impl NativeSpell {
    /// The system's dictionaries, checked on the process's shared worker, in the locale's
    /// language (`LC_ALL`, else `LANG`).
    pub fn system() -> NativeSpell {
        let jobs = SYSTEM
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let locale = locale_lang(
            std::env::var("LC_ALL").ok().as_deref(),
            std::env::var("LANG").ok().as_deref(),
        );
        NativeSpell {
            jobs,
            languages: locale.into_iter().collect(),
        }
    }

    /// `config`'s dictionaries on a worker of their own, in `languages` by default.
    pub fn with_config(config: SpellConfig, languages: Vec<Lang>) -> NativeSpell {
        NativeSpell {
            jobs: start(config),
            languages,
        }
    }

    /// Send a job whose answer comes back on a oneshot; `fallback` when the worker is gone.
    fn ask<T: 'static>(
        &self,
        job: impl FnOnce(oneshot::Sender<T>) -> Job,
        fallback: T,
    ) -> SpellFuture<T> {
        let (reply, answer) = oneshot::channel();
        let sent = self.jobs.send(job(reply)).is_ok();
        Box::pin(async move {
            match sent {
                true => answer.await.unwrap_or(fallback),
                false => fallback,
            }
        })
    }
}

impl SpellService for NativeSpell {
    fn languages(&self) -> Vec<Lang> {
        self.languages.clone()
    }

    fn paragraphs(&self, surface: &MountedData) -> Probe<Vec<Paragraph>> {
        let Some(handle) = surface.downcast::<NodeHandle>() else {
            return Probe::Unknown;
        };
        let Some(doc) = handle.try_doc() else {
            return Probe::Busy;
        };
        match doc.get_node(handle.node_id()) {
            Some(_) => Probe::Found(paragraphs(&doc, handle.node_id())),
            None => Probe::Unknown,
        }
    }

    fn check(&self, langs: Vec<Lang>, words: Vec<String>) -> SpellFuture<Vec<String>> {
        self.ask(
            |reply| Job::Check {
                langs,
                words,
                reply,
            },
            Vec::new(),
        )
    }

    fn suggest(&self, langs: Vec<Lang>, word: String) -> SpellFuture<Vec<String>> {
        self.ask(|reply| Job::Suggest { langs, word, reply }, Vec::new())
    }

    fn ignore(&self, word: String) {
        let _ = self.jobs.send(Job::Ignore(word));
    }

    fn learn(&self, lang: Lang, word: String) -> SpellFuture<Learned> {
        self.ask(
            |reply| Job::Learn { lang, word, reply },
            Learned::SessionOnly,
        )
    }
}
