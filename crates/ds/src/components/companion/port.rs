//! The seam an app's companion client installs: how a summon reaches a field and how an answer
//! comes back (design/32 section 5). `docket-ds` implements it in the docket repo.

use super::answer::model::AnswerView;
use dioxus::prelude::*;
use ds_intents::{ContextChip, SummonAnswerMark, SummonSerial};

/// Installed by the app's companion client; [`NoPort`] stands in where the feature is absent.
pub trait CompanionPort: 'static {
    /// The field `ds` found focused answers a summon synchronously through `handler`.
    fn on_summon(&self, handler: Callback<SummonSerial, SummonAnswerMark>);

    /// Sends the prompt with the chips the person kept.
    fn ask(&self, serial: SummonSerial, prompt: String, chips: Vec<ContextChip>);

    /// The answers for each serial, as props: the port converts the wire to [`AnswerView`].
    fn answers(&self) -> ReadSignal<Option<(SummonSerial, AnswerView)>>;

    /// Drops the ask in flight for `serial`.
    fn cancel(&self, serial: SummonSerial);
}

/// The port of an app with no companion client: no summon is taken, and no answer ever comes.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoPort;

impl CompanionPort for NoPort {
    fn on_summon(&self, _handler: Callback<SummonSerial, SummonAnswerMark>) {}

    fn ask(&self, _serial: SummonSerial, _prompt: String, _chips: Vec<ContextChip>) {}

    fn answers(&self) -> ReadSignal<Option<(SummonSerial, AnswerView)>> {
        todo!("NoPort::answers: a signal that never changes, made in the caller's Dioxus scope")
    }

    fn cancel(&self, _serial: SummonSerial) {}
}
