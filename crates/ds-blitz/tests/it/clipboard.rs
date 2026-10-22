//! Copy and paste. Ctrl+C on a selection in one field and Ctrl+V into
//! another move the text through the harness's in-memory clipboard, and the app's own
//! `ds_blitz::clipboard::{write_text, read_text}` reach the same clipboard.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_blitz::clipboard::{
    ClipboardError, ClipboardTarget, read_from, read_text, write_text, write_to,
};
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 260,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} missing:\n{}", harness.html()))
}

/// Two fields, each echoing its value below, and two buttons that copy and paste through the
/// app's own clipboard calls.
#[allow(non_snake_case)]
fn Fields() -> Element {
    let mut from = use_signal(|| "hello quire".to_owned());
    let mut to = use_signal(String::new);
    let mut pasted = use_signal(String::new);
    let mut primary = use_signal(String::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "display:flex; flex-direction:column; gap:12px; width:300px; padding:12px",
                div { id: "from", style: "display:flex",
                    TextField { label: "From", value: from(),
                        oninput: move |value| from.set(value) }
                }
                div { id: "to", style: "display:flex",
                    TextField { label: "To", value: to(),
                        oninput: move |value| to.set(value) }
                }
                div { id: "copy", style: "display:flex",
                    Button { label: "Copy address",
                        onclick: move |_| { let _ = write_text("ada@example.org"); } }
                }
                div { id: "select", style: "display:flex",
                    Button { label: "Select address",
                        onclick: move |_| { let _ = write_to(ClipboardTarget::Primary, "primary text"); } }
                }
                div { id: "paste-primary", style: "display:flex",
                    Button { label: "Paste selection",
                        onclick: move |_| primary.set(read_from(ClipboardTarget::Primary).unwrap_or_else(|e| format!("error: {e:?}"))) }
                }
                div { id: "paste", style: "display:flex",
                    Button { label: "Paste",
                        onclick: move |_| pasted.set(read_text().unwrap_or_else(|e| format!("error: {e:?}"))) }
                }
            }
            p { class: "to", {to()} }
            p { class: "pasted", {pasted()} }
            p { class: "primary", {primary()} }
        }
    }
}

#[test]
fn ctrl_c_on_a_selection_then_ctrl_v_into_a_second_field() {
    let mut harness = Harness::new(Fields, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let from = centre(&harness, "#from input");
    harness.send(Input::click(from));
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('a')));
    assert_eq!(
        harness.selected_text("#from input").as_deref(),
        Some("hello quire")
    );
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('c')));
    assert_eq!(harness.clipboard_text().as_deref(), Some("hello quire"));

    let to = centre(&harness, "#to input");
    harness.send(Input::click(to));
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('v')));
    harness.advance(ms(20));
    assert_eq!(harness.text_of(".to").as_deref(), Some("hello quire"));
}

#[test]
fn the_apps_write_and_read_reach_the_same_clipboard() {
    let mut harness = Harness::new(Fields, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let copy = centre(&harness, "#copy .ds-button");
    harness.send(Input::click(copy));
    assert_eq!(harness.clipboard_text().as_deref(), Some("ada@example.org"));

    harness.set_clipboard_text("from another app");
    let paste = centre(&harness, "#paste .ds-button");
    harness.send(Input::click(paste));
    assert_eq!(
        harness.text_of(".pasted").as_deref(),
        Some("from another app")
    );
}

#[test]
fn outside_a_document_there_is_no_clipboard() {
    assert_eq!(read_text(), Err(ClipboardError::NoHost));
}

#[test]
fn the_primary_selection_and_the_clipboard_are_separate_slots() {
    let mut harness = Harness::new(Fields, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let copy = centre(&harness, "#copy .ds-button");
    harness.send(Input::click(copy));
    assert_eq!(
        harness.primary_text(),
        None,
        "a copy leaves the selection alone"
    );

    let select = centre(&harness, "#select .ds-button");
    harness.send(Input::click(select));
    assert_eq!(harness.primary_text().as_deref(), Some("primary text"));
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("ada@example.org"),
        "a selection write leaves the clipboard alone"
    );

    let paste = centre(&harness, "#paste-primary .ds-button");
    harness.send(Input::click(paste));
    assert_eq!(harness.text_of(".primary").as_deref(), Some("primary text"));
}

mod files {
    use ds_blitz::clipboard::{Clipboard, ClipboardFiles, FileOp, Memory};
    use ds_blitz::seam::MemoryShell;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn memory() -> (Arc<MemoryShell>, Memory) {
        let shell = Arc::new(MemoryShell::default());
        (Arc::clone(&shell), Memory(shell))
    }

    fn odd_paths() -> Vec<PathBuf> {
        ["/home/ada/My Photos/日本 100%.png", "/tmp/a#b?c.txt"]
            .map(PathBuf::from)
            .to_vec()
    }

    #[test]
    fn names_with_spaces_non_ascii_and_percent_round_trip() {
        let files = ClipboardFiles::new(FileOp::Copy, odd_paths());
        let list = files.to_uri_list();
        assert!(list.starts_with("file:///home/ada/My%20Photos/%E6%97%A5"));
        assert!(list.contains("100%25.png"));
        assert_eq!(ClipboardFiles::from_uri_list(&list), Some(files));
    }

    #[test]
    fn a_copy_is_written_in_both_formats_and_read_back() {
        let (shell, clipboard) = memory();
        let files = ClipboardFiles::new(FileOp::Copy, odd_paths());
        clipboard.write_files(&files).unwrap();
        assert!(shell.gnome_copied().unwrap().starts_with("copy\nfile:///"));
        assert!(shell.uri_list().unwrap().starts_with("file:///"));
        assert_eq!(clipboard.read_files(), Some(files));
    }

    #[test]
    fn a_cut_from_another_app_reads_as_cut() {
        let (shell, clipboard) = memory();
        shell.put_gnome_copied("cut\nfile:///tmp/a%20b.txt\nfile:///tmp/c.txt".to_owned());
        assert_eq!(
            clipboard.read_files(),
            Some(ClipboardFiles::new(
                FileOp::Cut,
                ["/tmp/a b.txt", "/tmp/c.txt"]
            ))
        );
    }

    #[test]
    fn a_bare_uri_list_is_a_copy_and_skips_comments_and_remote_uris() {
        let (shell, clipboard) = memory();
        shell.put_uri_list(
            "# copied\r\nfile://localhost/tmp/x.txt\r\nhttps://example.org/y\r\nfile://other/z\r\n"
                .to_owned(),
        );
        assert_eq!(
            clipboard.read_files(),
            Some(ClipboardFiles::new(FileOp::Copy, ["/tmp/x.txt"]))
        );
    }

    #[test]
    fn plain_text_of_paths_is_files_and_other_text_is_not() {
        let (shell, clipboard) = memory();
        shell.put("/tmp/a.txt\nfile:///tmp/b%20c.txt\n".to_owned());
        assert_eq!(
            clipboard.read_files(),
            Some(ClipboardFiles::new(
                FileOp::Copy,
                ["/tmp/a.txt", "/tmp/b c.txt"]
            ))
        );
        shell.put("hello quire".to_owned());
        assert_eq!(clipboard.read_files(), None);
    }

    #[test]
    fn a_text_copy_replaces_the_files_and_a_bad_first_line_is_refused() {
        let (shell, clipboard) = memory();
        clipboard
            .write_files(&ClipboardFiles::new(FileOp::Cut, ["/tmp/a"]))
            .unwrap();
        clipboard.write_text("words").unwrap();
        assert_eq!(clipboard.read_files(), None);
        assert_eq!(shell.gnome_copied(), None);
        assert_eq!(
            ClipboardFiles::from_gnome_copied("move\nfile:///tmp/a"),
            None
        );
    }

    #[test]
    fn the_app_matches_a_paste_to_its_cut_set_in_any_order() {
        let cut = ClipboardFiles::new(FileOp::Cut, ["/tmp/a", "/tmp/b"]);
        assert!(cut.same_files(&[PathBuf::from("/tmp/b"), PathBuf::from("/tmp/a")]));
        assert!(!cut.same_files(&[PathBuf::from("/tmp/a")]));
    }
}
