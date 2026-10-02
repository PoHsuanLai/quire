//! The seam an app's companion client installs: how a summon reaches a field and how an answer
//! comes back (design/32 section 5). `docket-ds` implements it in the docket repo.

use super::answer::model::AnswerView;
use dioxus::prelude::*;
use ds_intents::{
    ContextChip, DictateSerial, HeardMark, SummonAnswerMark, SummonOriginMark, SummonSerial,
};

/// Installed by the app's companion client; [`NoPort`] stands in where the feature is absent.
pub trait CompanionPort: 'static {
    /// The field `ds` found focused answers a summon synchronously through `handler`. The origin
    /// says how it was started: a voice summon fills the prompt from [`CompanionPort::heard`].
    fn on_summon(&self, handler: Callback<(SummonSerial, SummonOriginMark), SummonAnswerMark>);

    /// Sends the prompt with the chips the person kept.
    fn ask(&self, serial: SummonSerial, prompt: String, chips: Vec<ContextChip>);

    /// The answers for each serial, as props: the port converts the wire to [`AnswerView`].
    fn answers(&self) -> ReadSignal<Option<(SummonSerial, AnswerView)>>;

    /// Drops the ask in flight for `serial`.
    fn cancel(&self, serial: SummonSerial);

    /// What a summoned prompt hears while the person speaks: the level, the tail, the committed
    /// segments and the end.
    fn heard(&self) -> ReadSignal<Option<(SummonSerial, HeardMark)>>;
}

/// Installed by the app's dictation client; [`NoDictation`] stands in where the feature is absent.
pub trait DictationPort: 'static {
    /// The focused field answers a dictation request through `handler`.
    fn on_dictate(&self, handler: Callback<DictateSerial, SummonAnswerMark>);

    /// What the focused field hears for each dictation.
    fn text(&self) -> ReadSignal<Option<(DictateSerial, HeardMark)>>;

    /// Stops the dictation `serial` (the field ended it).
    fn stop(&self, serial: DictateSerial);
}

/// The port of an app with no companion client: no summon is taken, and no answer ever comes.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoPort;

impl CompanionPort for NoPort {
    fn on_summon(&self, _handler: Callback<(SummonSerial, SummonOriginMark), SummonAnswerMark>) {}

    fn ask(&self, _serial: SummonSerial, _prompt: String, _chips: Vec<ContextChip>) {}

    fn answers(&self) -> ReadSignal<Option<(SummonSerial, AnswerView)>> {
        todo!("NoPort::answers: a signal that never changes, made in the caller's Dioxus scope")
    }

    fn cancel(&self, _serial: SummonSerial) {}

    fn heard(&self) -> ReadSignal<Option<(SummonSerial, HeardMark)>> {
        todo!("NoPort::heard: a signal that never changes, made in the caller's Dioxus scope")
    }
}

/// The dictation port of an app with no dictation client: no dictation is taken.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoDictation;

impl DictationPort for NoDictation {
    fn on_dictate(&self, _handler: Callback<DictateSerial, SummonAnswerMark>) {}

    fn text(&self) -> ReadSignal<Option<(DictateSerial, HeardMark)>> {
        todo!("NoDictation::text: a signal that never changes, made in the caller's Dioxus scope")
    }

    fn stop(&self, _serial: DictateSerial) {}
}
