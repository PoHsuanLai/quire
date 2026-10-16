//! The missing-helper sheet as markup: every phase in light and dark matches its golden under
//! `tests/snapshots/helpers/`, lints clean, and uses only `ds-` classes the stylesheet styles.
//!
//! `DS_BLESS=1 cargo test -p ds-shell --test it helpers_ssr` rewrites the goldens.

use crate::golden;
use crate::hygiene;

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::prelude::*;
use ds_lint::LintConfig;
use ds_shell::accounts::model::CopyState;
use ds_shell::helpers::model::HelperPhase;
use ds_shell::prelude::*;

#[derive(Clone, PartialEq)]
struct StageProps {
    theme: Theme,
    specimen: usize,
}

#[allow(non_snake_case)]
fn Stage(props: StageProps) -> Element {
    let body = (SPECIMENS[props.specimen].1)();
    rsx! {
        Ds {
            appearance: Appearance { theme: props.theme, ..Appearance::default() },
            material: Material::Sheet,
            extent: RootExtent::Viewport,
            stylesheet: Inject::Host,
            {body}
        }
    }
}

fn sheet(phase: HelperPhase) -> Element {
    rsx! {
        HelperSheet {
            app: "Anyview",
            tool: "mpv",
            purpose: "play videos",
            phase,
            on_install: |_| {},
            on_dismiss: |_| {},
        }
    }
}

type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("ask", || sheet(HelperPhase::Ask)),
    ("installing", || sheet(HelperPhase::Installing)),
    ("failed", || {
        sheet(HelperPhase::Failed {
            reason: "The network is unreachable, so the package could not be downloaded.".into(),
        })
    }),
    ("not-found", || {
        sheet(HelperPhase::NotFound {
            package: "mpv".into(),
        })
    }),
    ("unsupported", || {
        sheet(HelperPhase::Unsupported {
            program: "mpv".into(),
            command: None,
        })
    }),
    ("unsupported-command", || {
        sheet(HelperPhase::Unsupported {
            program: "mpv".into(),
            command: Some("sudo pacman -S mpv".into()),
        })
    }),
    ("unsupported-copied", || {
        rsx! {
            HelperSheet {
                app: "Anyview",
                tool: "mpv",
                purpose: "play videos",
                phase: HelperPhase::Unsupported {
                    program: "mpv".into(),
                    command: Some("sudo pacman -S mpv".into()),
                },
                on_install: |_| {},
                on_dismiss: |_| {},
                copy: CopyState::Copied,
            }
        }
    }),
];

fn render(theme: Theme, specimen: usize) -> String {
    let mut dom = VirtualDom::new_with_props(Stage, StageProps { theme, specimen });
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

fn light(name: &str) -> String {
    let at = SPECIMENS.iter().position(|(known, _)| *known == name);
    render(
        Theme::Light,
        at.unwrap_or_else(|| panic!("no specimen {name}")),
    )
}

#[test]
fn every_phase_matches_its_golden_in_light_and_dark() {
    let failures: Vec<String> = SPECIMENS
        .iter()
        .enumerate()
        .flat_map(|(at, (name, _))| {
            [("light", Theme::Light), ("dark", Theme::Dark)]
                .into_iter()
                .filter_map(move |(look, theme)| {
                    golden::check(&format!("helpers/{name}-{look}.html"), &render(theme, at)).err()
                })
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Lint offences and unstyled classes of every specimen, swept by `golden_hygiene`.
pub(crate) fn hygiene_failures() -> Vec<String> {
    let sheet = ds_shell::stylesheet();
    let config = LintConfig::new(&ds_shell::kits());
    SPECIMENS
        .iter()
        .enumerate()
        .flat_map(|(at, (name, _))| {
            hygiene::failures(name, &render(Theme::Light, at), sheet, &config)
        })
        .collect()
}

#[test]
fn the_question_says_what_the_app_needs_and_offers_both_answers() {
    let html = light("ask");
    assert!(html.contains("Anyview needs mpv to play videos."), "{html}");
    assert!(
        html.contains("Not Now") && html.contains("Install\u{2026}"),
        "{html}"
    );
    // As NSAlert: the default comes first in the row-reverse footer, so it is drawn rightmost.
    let install = html.find("Install\u{2026}").unwrap_or(usize::MAX);
    let not_now = html.find("Not Now").unwrap_or(0);
    assert!(install < not_now, "Install… must precede Not Now: {html}");
}

#[test]
fn installing_offers_nothing_to_press() {
    let html = light("installing");
    assert!(html.contains("Installing mpv"), "{html}");
    assert!(!html.contains("<button"), "{html}");
}

#[test]
fn the_end_phases_close_and_say_what_to_look_for() {
    for (name, needle) in [
        ("failed", "network"),
        ("not-found", "\u{201c}mpv\u{201d}"),
        ("unsupported", "provides"),
    ] {
        let html = light(name);
        assert!(
            html.contains("Close") && html.contains(needle),
            "{name}: {html}"
        );
        assert!(!html.contains("Install\u{2026}"), "{name}");
    }
}

#[test]
fn an_unsupported_system_with_a_command_shows_it_selectable_with_a_copy_button() {
    let html = light("unsupported-command");
    assert!(
        html.contains("Run this in Terminal to install it yourself, then try again:"),
        "{html}"
    );
    assert!(
        html.contains("<span class=\"ds-command-line-text\">sudo pacman -S mpv</span>"),
        "{html}"
    );
    assert!(
        html.contains("<span class=\"ds-button-label\">Copy</span>"),
        "{html}"
    );
    assert!(
        html.contains("Close") && !html.contains("Install\u{2026}"),
        "{html}"
    );
    let copied = light("unsupported-copied");
    assert!(
        copied.contains("<span class=\"ds-button-label\">Copied</span>")
            && !copied.contains(">Copy<"),
        "the host's word that it copied turns the label to Copied: {copied}"
    );
    let styled = ds_shell::stylesheet().lines().any(|rule| {
        rule.starts_with(".ds-command-line-text{") && rule.contains("user-select:text")
    });
    assert!(styled, "the command line must be selectable");
}

#[test]
fn an_unsupported_system_without_a_command_has_no_command_line() {
    let html = light("unsupported");
    assert!(html.contains("then try again."), "{html}");
    assert!(
        !html.contains("ds-command-line") && !html.contains("Copy"),
        "{html}"
    );
}
