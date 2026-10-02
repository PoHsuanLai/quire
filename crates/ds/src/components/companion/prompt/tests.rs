use super::host::{PromptHost, prompt_host};
use crate::components::fields::text_field_model::FieldKind;
use ds_core::word::Word;

#[test]
fn prompt_host_is_total() {
    const CASES: &[(FieldKind, PromptHost)] = &[
        (FieldKind::Plain, PromptHost::Field),
        (FieldKind::Search, PromptHost::Field),
        (FieldKind::Multiline, PromptHost::Anchored),
        (FieldKind::Secure, PromptHost::Refused),
    ];
    assert_eq!(CASES.len(), FieldKind::ALL.len());
    for &(kind, host) in CASES {
        assert_eq!(prompt_host(kind), host, "{kind:?}");
    }
}
