//! One function per question. Each builds a small document, drives it, renders PNGs and
//! reads a few pixels back so the table carries a machine-checked hint next to the PNG.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use base64::Engine as _;
use dioxus::prelude::*;
use futures_timer::Delay;
use keyboard_types::{Code, Key};

use crate::harness::{self, CountingWaker, Net, Shot, W, build, click, hue, key, shot};

pub struct Row {
    pub id: &'static str,
    pub verdict: String,
    pub pngs: Vec<String>,
    pub seen: String,
}

fn row(id: &'static str, verdict: &str, shots: &[&Shot], seen: String) -> Row {
    Row {
        id,
        verdict: verdict.to_string(),
        pngs: shots.iter().map(|s| s.path()).collect(),
        seen,
    }
}

fn yes_no(ok: bool) -> &'static str {
    if ok { "YES" } else { "NO" }
}

/// Width of the run of non-white pixels starting at x=0 on row y.
fn run_width(s: &Shot, y: u32) -> u32 {
    (0..W).take_while(|&x| hue(s.px(x, y)) != "white").count() as u32
}

/// Horizontal extent (first..last non-white pixel) on row y.
fn ink_extent(s: &Shot, y0: u32, y1: u32) -> u32 {
    let inked = |x: u32| (y0..y1).any(|y| s.px(x, y)[0] < 128);
    let first = (0..W).find(|&x| inked(x));
    let last = (0..W).rev().find(|&x| inked(x));
    match (first, last) {
        (Some(a), Some(b)) => b - a,
        _ => 0,
    }
}

// ---------------------------------------------------------------- S1
const S1_CSS: &str =
    "body { margin: 0 } .s1 { width: 200px; height: 100px; background-color: rgb(0,160,0); }";

fn s1_app() -> Element {
    rsx! {
        style { {S1_CSS} }
        div { class: "s1" }
        p { "S1: the box above is green only if a <style> in the body applies." }
    }
}

pub fn s1() -> Row {
    let mut doc = build(s1_app, None, Net::Dummy);
    let s = shot(&mut doc, 0.0, "s01-body-style");
    let c = hue(s.px(100, 50));
    row(
        "S1",
        yes_no(c == "green"),
        &[&s],
        format!("box at (100,50) is {c}"),
    )
}

// ---------------------------------------------------------------- S2
const S2_CSS: &str = "
body { margin: 0 }
.ds { --paper: rgb(240,240,240); --ink: rgb(20,20,20); background-color: var(--paper); color: var(--ink); padding: 10px; }
.ds[data-theme=dark] { --paper: rgb(20,20,20); --ink: rgb(240,240,240); }
.ds[data-accent=blue] { --accent: rgb(0,0,220); }
.ds[data-accent=red] { --accent: rgb(220,0,0); }
.chip { background-color: var(--accent); width: 80px; height: 30px; }
.frame { background-color: var(--f-x); width: 80px; height: 30px; }
";

static S2_VARIANT: AtomicU8 = AtomicU8::new(0);
const S2_VARIANTS: [&str; 3] = ["attr", "nswild", "class"];

/// The same cascade with three selector spellings: `[data-x=v]`, `[*|data-x=v]`, `.x-v`.
fn s2_css() -> String {
    match S2_VARIANT.load(Ordering::SeqCst) {
        0 => S2_CSS.to_string(),
        1 => S2_CSS.replace("[data-", "[*|data-"),
        _ => S2_CSS
            .replace("[data-theme=dark]", ".theme-dark")
            .replace("[data-accent=blue]", ".accent-blue")
            .replace("[data-accent=red]", ".accent-red"),
    }
}

fn s2_app() -> Element {
    let css = s2_css();
    rsx! {
        style { {css} }
        div { class: "ds theme-light accent-blue", "data-theme": "light", "data-accent": "blue", style: "--f-x: rgb(220,0,220)",
            div { class: "chip" }
            div { class: "frame" }
            div { class: "ds theme-dark accent-red", "data-theme": "dark", "data-accent": "red",
                div { class: "chip" }
                div { class: "frame" }
                "nested .ds: dark paper, light ink, red accent"
            }
            "outer .ds: light paper, blue accent"
        }
    }
}

pub fn s2() -> Row {
    let mut shots = Vec::new();
    let mut seen = Vec::new();
    let mut oks = Vec::new();
    for (i, name) in S2_VARIANTS.iter().enumerate() {
        S2_VARIANT.store(i as u8, Ordering::SeqCst);
        let mut doc = build(s2_app, None, Net::Dummy);
        let s = shot(&mut doc, 0.0, &format!("s02-ds-cascade-{name}"));
        let got = [
            ("outer paper", hue(s.px(300, 5)), "white"),
            ("outer chip", hue(s.px(50, 25)), "blue"),
            ("outer inline --f-x", hue(s.px(50, 55)), "magenta"),
            ("nested paper", hue(s.px(300, 100)), "black"),
            ("nested chip", hue(s.px(60, 95)), "red"),
            ("nested inherits --f-x", hue(s.px(60, 125)), "magenta"),
        ];
        let ok = got.iter().all(|(_, g, want)| g == want);
        let bad: Vec<String> = got
            .iter()
            .filter(|(_, g, want)| g != want)
            .map(|(what, g, want)| format!("{what}={g}/want {want}"))
            .collect();
        seen.push(format!(
            "{name}: {}",
            if ok {
                "all 6 match".to_string()
            } else {
                bad.join(", ")
            }
        ));
        oks.push(ok);
        shots.push(s);
    }
    let verdict = match (oks[0], oks[1] || oks[2]) {
        (true, _) => "YES",
        (false, true) => "PARTIAL",
        _ => "NO",
    };
    row(
        "S2",
        verdict,
        &shots.iter().collect::<Vec<_>>(),
        seen.join("; "),
    )
}

// ---------------------------------------------------------------- S3
const S3_CSS: &str = "
body { margin: 0 }
@keyframes s3lit { from { transform: translateX(0px) } to { transform: translateX(300px) } }
@keyframes s3var { from { background-color: var(--from) } to { background-color: var(--to) } }
@keyframes s3dist { from { transform: translateX(0px) } to { transform: translateX(var(--dist)) } }
.b { width: 40px; height: 40px; margin-bottom: 10px; background-color: rgb(0,160,0); }
.a1 { animation: s3lit 1s linear both; }
.a2 { --from: rgb(255,0,0); --to: rgb(0,0,255); animation: s3var 1s linear both; }
.a3 { --dist: 300px; animation: s3dist 1s linear both; }
.a4 { --t: 1s; animation: s3lit var(--t) linear both; }
";

fn s3_app() -> Element {
    rsx! {
        style { {S3_CSS} }
        div { class: "b a1" }
        div { class: "b a2" }
        div { class: "b a3" }
        div { class: "b a4" }
    }
}

/// x of the first non-white pixel on row y: where a translated 40px box starts.
fn box_x(s: &Shot, y: u32) -> Option<u32> {
    (0..W).find(|&x| hue(s.px(x, y)) != "white")
}

pub fn s3() -> Row {
    let mut doc = build(s3_app, None, Net::Dummy);
    let shots: Vec<Shot> = [0.0, 0.1, 0.3]
        .iter()
        .map(|&t| {
            shot(
                &mut doc,
                t,
                &format!("s03-keyframes-t{:03}", (t * 100.0) as u32),
            )
        })
        .collect();
    let last = &shots[2];
    let lit = box_x(last, 20);
    let colour = |s: &Shot| s.px(20, 70);
    let dist = box_x(last, 120);
    let dur = box_x(last, 170);
    let lit_ok = lit.is_some_and(|x| (80..=100).contains(&x));
    let var_ok = colour(&shots[0]) != colour(last) && hue(colour(&shots[0])) == "red";
    let dist_ok = dist.is_some_and(|x| (80..=100).contains(&x));
    let dur_ok = dur.is_some_and(|x| (80..=100).contains(&x));
    let verdict = match (lit_ok, var_ok && dist_ok && dur_ok) {
        (true, true) => "YES",
        (true, false) => "PARTIAL",
        _ => "NO",
    };
    let seen = format!(
        "t=0.3: literal keyframes box x={lit:?} (want ~90); var() colour t0={:?} t0.3={:?}; \
         translateX(var(--dist)) x={dist:?}; duration var(--t) x={dur:?}",
        colour(&shots[0]),
        colour(last)
    );
    row("S3", verdict, &shots.iter().collect::<Vec<_>>(), seen)
}

// ---------------------------------------------------------------- S4 / S5 phase
static PHASE: AtomicU8 = AtomicU8::new(0);

fn phase() -> u8 {
    PHASE.load(Ordering::SeqCst)
}

// ---------------------------------------------------------------- S4
const S4_CSS: &str = "
body { margin: 0 }
.t { width: var(--w); height: 40px; background-color: var(--c); margin-bottom: 10px;
     transition: width 1s linear, background-color 1s linear; }
.t.on-a { --w: 40px; --c: rgb(255,0,0); }
.t.on-b { --w: 340px; --c: rgb(0,0,255); }
.u { width: 40px; height: 40px; background-color: rgb(0,160,0); transform: translateX(var(--x));
     transition: transform 1s linear; }
.u.on-a { --x: 0px; }
.u.on-b { --x: 300px; }
";

fn s4_app() -> Element {
    let on = if phase() == 0 { "on-a" } else { "on-b" };
    rsx! {
        style { {S4_CSS} }
        div { class: "t {on}" }
        div { class: "u {on}" }
    }
}

pub fn s4() -> Row {
    PHASE.store(0, Ordering::SeqCst);
    let mut doc = build(s4_app, None, Net::Dummy);
    let before = shot(&mut doc, 0.0, "s04-transition-before");
    PHASE.store(1, Ordering::SeqCst);
    harness::rerender(&mut doc);
    let t0 = shot(&mut doc, 1.0, "s04-transition-t000");
    let t1 = shot(&mut doc, 1.1, "s04-transition-t010");
    let t3 = shot(&mut doc, 1.3, "s04-transition-t030");
    let w = run_width(&t3, 20);
    let x = box_x(&t3, 70);
    let mid_colour = t3.px(10, 20);
    let width_ok = (100..=180).contains(&w);
    let tf_ok = x.is_some_and(|x| (70..=110).contains(&x));
    let verdict = match (width_ok, tf_ok) {
        (true, true) => "YES",
        (false, false) => "NO",
        _ => "PARTIAL",
    };
    let seen = format!(
        "0.3 s after flipping data-on: width={w} (want ~130, 340 = jumped), colour={mid_colour:?}, \
         translateX(var) box x={x:?} (want ~90); widths t0={} t0.1={}",
        run_width(&t0, 20),
        run_width(&t1, 20)
    );
    row("S4", verdict, &[&before, &t0, &t1, &t3], seen)
}

// ---------------------------------------------------------------- S5
const S5_CSS: &str = "
body { margin: 0 }
@keyframes grow { from { width: 20px } to { width: 380px } }
@keyframes grow--b { from { width: 20px } to { width: 380px } }
.r { height: 40px; margin-bottom: 10px; background-color: rgb(0,160,0);
     animation-duration: 1s; animation-timing-function: linear; animation-fill-mode: both; }
.r.pulse-a { animation-name: grow; }
.r.pulse-b { animation-name: grow--b; }
.r.off { animation-name: none; }
";

fn s5_app() -> Element {
    let p = phase();
    let swap = if p == 0 { "r pulse-a" } else { "r pulse-b" };
    let removal = if p == 1 { "r pulse-a off" } else { "r pulse-a" };
    rsx! {
        style { {S5_CSS} }
        div { class: swap }
        div { class: "r pulse-a" }
        div { class: removal }
    }
}

pub fn s5() -> Row {
    PHASE.store(0, Ordering::SeqCst);
    let mut doc = build(s5_app, None, Net::Dummy);
    let _ = shot(&mut doc, 0.0, "s05-restart-t000");
    let mid = shot(&mut doc, 0.5, "s05-restart-t050");
    // Phase 1: row 1 swaps grow -> grow--b; row 3 drops its animation for one resolve.
    PHASE.store(1, Ordering::SeqCst);
    harness::rerender(&mut doc);
    harness::resolve(&mut doc, 0.5);
    // Phase 2: row 3 gets its animation back.
    PHASE.store(2, Ordering::SeqCst);
    harness::rerender(&mut doc);
    let after = shot(&mut doc, 0.6, "s05-restart-t060");
    let widths = |s: &Shot| [20, 70, 120].map(|y| run_width(s, y));
    let [swap, control, removal] = widths(&after);
    let swap_ok = swap < 100;
    let removal_ok = removal < 100;
    let verdict = match (swap_ok, removal_ok) {
        (true, _) => "YES",
        (false, true) => "NO (fallback works)",
        _ => "NO",
    };
    let seen = format!(
        "widths at t=0.5 {:?}; 0.1 s after the change: name swap={swap}, control={control}, \
         one-frame removal={removal} (restart => ~56, continue => ~236)",
        widths(&mid)
    );
    row("S5", verdict, &[&mid, &after], seen)
}

// ---------------------------------------------------------------- S6
const S6_CSS: &str = "
body { margin: 0 }
svg { display: block; }
.red { color: rgb(220,0,0); }
.blue { color: rgb(0,0,220); }
.css-stroke path { stroke: currentColor; }
";

fn s6_app() -> Element {
    rsx! {
        style { {S6_CSS} }
        div { class: "red",
            svg { width: "100", height: "100", view_box: "0 0 24 24", fill: "none",
                stroke: "currentColor", stroke_width: "4",
                path { d: "M2 12h20M12 2v20" }
            }
        }
        div { class: "blue",
            svg { width: "100", height: "100", view_box: "0 0 24 24", fill: "none",
                stroke: "currentColor", stroke_width: "4",
                path { d: "M2 12h20M12 2v20" }
            }
        }
        div { class: "blue css-stroke", style: "position: absolute; left: 200px; top: 0px",
            svg { width: "100", height: "100", view_box: "0 0 24 24", fill: "none", stroke_width: "4",
                path { d: "M2 12h20M12 2v20" }
            }
        }
    }
}

pub fn s6() -> Row {
    let mut doc = build(s6_app, None, Net::Dummy);
    let s = shot(&mut doc, 0.0, "s06-svg-currentcolor");
    let a = hue(s.px(50, 50));
    let b = hue(s.px(50, 150));
    let c = hue(s.px(250, 50));
    let ok = a == "red" && b == "blue";
    row(
        "S6",
        yes_no(ok),
        &[&s],
        format!(
            "attr stroke=currentColor under red: {a}; under blue: {b}; CSS stroke:currentColor: {c}"
        ),
    )
}

// ---------------------------------------------------------------- S7
const S7_CSS: &str = "
body { margin: 0 }
.m { width: 100px; height: 100px; background-color: currentColor; color: rgb(0,160,0);
     mask-image: url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Ccircle cx='12' cy='12' r='6' fill='black'/%3E%3C/svg%3E\");
     mask-size: 100% 100%; mask-repeat: no-repeat; }
";

fn s7_app() -> Element {
    rsx! {
        style { {S7_CSS} }
        div { class: "m" }
        p { "mask-image data:svg, background currentColor (green disc on white = works)" }
    }
}

fn mask_verdict(s: &Shot) -> (bool, String) {
    let centre = hue(s.px(50, 50));
    let corner = hue(s.px(5, 5));
    (
        centre == "green" && corner == "white",
        format!("centre={centre} corner={corner}"),
    )
}

pub fn s7() -> Row {
    let mut dummy = build(s7_app, None, Net::Dummy);
    let a = shot(&mut dummy, 0.0, "s07-mask-svg-dummynet");
    let mut data = build(s7_app, None, Net::DataUri);
    let _ = shot(&mut data, 0.0, "s07-mask-svg-datauri-first");
    let b = shot(&mut data, 0.0, "s07-mask-svg-datauri");
    let (_, sa) = mask_verdict(&a);
    let (ok, sb) = mask_verdict(&b);
    row(
        "S7",
        yes_no(ok),
        &[&a, &b],
        format!("default net provider: {sa}; with a data: net provider: {sb}"),
    )
}

// ---------------------------------------------------------------- S8
static S8_CSS: OnceLock<String> = OnceLock::new();

fn checker_png() -> String {
    let mut img = image::RgbaImage::new(8, 8);
    for (x, y, p) in img.enumerate_pixels_mut() {
        *p = if (x < 4) == (y < 4) {
            image::Rgba([220, 0, 0, 255])
        } else {
            image::Rgba([0, 0, 220, 255])
        };
    }
    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .expect("encode png");
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn s8_app() -> Element {
    let css = S8_CSS.get_or_init(|| {
        format!(
            "body {{ margin: 0 }} .bg {{ width: 200px; height: 100px; \
             background-image: url(data:image/png;base64,{}); }}",
            checker_png()
        )
    });
    rsx! {
        style { {css.as_str()} }
        div { class: "bg" }
        p { "8x8 red/blue checker PNG tiled as background-image" }
    }
}

fn tile_verdict(s: &Shot) -> (bool, String) {
    let got = [(2, 2), (6, 2), (10, 2), (14, 6), (190, 90)].map(|(x, y)| hue(s.px(x, y)));
    (
        got == ["red", "blue", "red", "red", "blue"],
        format!("(2,2),(6,2),(10,2),(14,6),(190,90) = {got:?}"),
    )
}

pub fn s8() -> Row {
    let mut dummy = build(s8_app, None, Net::Dummy);
    let a = shot(&mut dummy, 0.0, "s08-bg-png-dummynet");
    let mut data = build(s8_app, None, Net::DataUri);
    let _ = shot(&mut data, 0.0, "s08-bg-png-datauri-first");
    let b = shot(&mut data, 0.0, "s08-bg-png-datauri");
    let (_, sa) = tile_verdict(&a);
    let (ok, sb) = tile_verdict(&b);
    row(
        "S8",
        yes_no(ok),
        &[&a, &b],
        format!("default net provider: {sa}; with a data: net provider: {sb}"),
    )
}

// ---------------------------------------------------------------- S9
static S9_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn log9(s: String) {
    S9_LOG.lock().unwrap().push(s);
}

const S9_CSS: &str = "
body { margin: 0 }
.target { position: absolute; left: 50px; top: 60px; width: 123px; height: 45px;
          background-color: rgb(0,160,0); color: white; }
.readout { position: absolute; left: 10px; top: 150px; font-size: 13px; }
";

fn s9_app() -> Element {
    let mut text = use_signal(|| "waiting for onmounted".to_string());
    rsx! {
        style { {S9_CSS} }
        div {
            class: "target",
            onmounted: move |e: MountedEvent| async move {
                let at_mount = e.get_client_rect().await;
                log9(format!("in handler: {at_mount:?}"));
                Delay::new(Duration::from_millis(20)).await;
                let later = e.get_client_rect().await;
                log9(format!("20 ms later: {later:?}"));
                text.set(format!("mount: {at_mount:?}\nlater: {later:?}"));
            },
            "target 123x45 @ (50,60)"
        }
        pre { class: "readout", "{text}" }
    }
}

pub fn s9() -> Row {
    S9_LOG.lock().unwrap().clear();
    let mut doc = build(s9_app, None, Net::Dummy);
    let waker = Arc::new(CountingWaker::default());
    harness::resolve(&mut doc, 0.0);
    harness::poll_with(&mut doc, &waker);
    std::thread::sleep(Duration::from_millis(60));
    harness::poll_with(&mut doc, &waker);
    let s = shot(&mut doc, 0.0, "s09-onmounted-rect");
    let log = S9_LOG.lock().unwrap().clone();
    let later_ok = log
        .iter()
        .any(|l| l.contains("20 ms later") && l.contains("123"));
    let at_mount_ok = log
        .iter()
        .any(|l| l.contains("in handler") && l.contains("123"));
    let verdict = match (at_mount_ok, later_ok) {
        (true, _) => "YES",
        (false, true) => "PARTIAL",
        _ => "NO",
    };
    row("S9", verdict, &[&s], log.join(" | "))
}

// ---------------------------------------------------------------- S10
const S10_CSS: &str = "
body { margin: 0 }
.flag { width: 200px; height: 40px; margin: 10px; color: white; }
.off { background-color: rgb(220,0,0); }
.on { background-color: rgb(0,160,0); }
.btn { position: absolute; left: 10px; top: 150px; width: 200px; height: 40px; }
";

fn s10_app() -> Element {
    let mut body = use_signal(|| false);
    let mut handler = use_signal(|| false);
    use_hook(|| {
        spawn(async move {
            Delay::new(Duration::from_millis(50)).await;
            body.set(true);
        })
    });
    let class = |on: bool| if on { "flag on" } else { "flag off" };
    rsx! {
        style { {S10_CSS} }
        div { class: class(body()), "timer spawned from body" }
        div { class: class(handler()), "timer spawned from handler" }
        button {
            class: "btn",
            onclick: move |_| {
                spawn(async move {
                    Delay::new(Duration::from_millis(50)).await;
                    handler.set(true);
                });
            },
            "click me"
        }
    }
}

pub fn s10() -> Row {
    let mut doc = build(s10_app, None, Net::Dummy);
    let waker = Arc::new(CountingWaker::default());
    harness::resolve(&mut doc, 0.0);
    let first = harness::poll_with(&mut doc, &waker);
    let w0 = waker.count();
    std::thread::sleep(Duration::from_millis(120));
    let body_wakes = waker.count() - w0;
    let body_poll = harness::poll_with(&mut doc, &waker);
    let a = shot(&mut doc, 0.0, "s10-timer-body");

    click(&mut doc, 100.0, 170.0);
    let click_poll = harness::poll_with(&mut doc, &waker);
    let w1 = waker.count();
    std::thread::sleep(Duration::from_millis(120));
    let handler_wakes = waker.count() - w1;
    let handler_poll = harness::poll_with(&mut doc, &waker);
    let b = shot(&mut doc, 0.0, "s10-timer-handler");

    let body_on = hue(b.px(100, 30)) == "green";
    let handler_on = hue(b.px(100, 90)) == "green";
    let verdict = match (body_on && body_wakes > 0, handler_on && handler_wakes > 0) {
        (true, true) => "YES (both)",
        (false, true) => "PARTIAL (handler only)",
        (true, false) => "PARTIAL (body only)",
        _ => "NO",
    };
    row(
        "S10",
        verdict,
        &[&a, &b],
        format!(
            "first poll={first}; body timer: wakes={body_wakes} poll={body_poll} box={}; \
             after click poll={click_poll}; handler timer: wakes={handler_wakes} poll={handler_poll} box={}",
            hue(a.px(100, 30)),
            hue(b.px(100, 90))
        ),
    )
}

// ---------------------------------------------------------------- S11
const S11_CSS: &str = "
body { margin: 0; padding: 10px; }
.f { font-family: 'Fira Mono', serif; font-size: 28px; line-height: 40px; height: 40px; }
";

fn s11_app() -> Element {
    rsx! {
        style { {S11_CSS} }
        div { class: "f", "iiiiiiiiii" }
        div { class: "f", "WWWWWWWWWW" }
        div { style: "font-size: 13px", "font-family: 'Fira Mono', serif  (mono => both lines equally wide)" }
    }
}

const FIRA: &[u8] = include_bytes!("../assets/FiraMono-Medium.ttf");

fn fira_ctx() -> (blitz_dom::FontContext, Vec<String>) {
    use parley::fontique::Blob;
    let mut ctx = blitz_dom::FontContext::new();
    let registered = ctx
        .collection
        .register_fonts(Blob::new(Arc::new(FIRA.to_vec())), None);
    let names = registered
        .iter()
        .filter_map(|(id, _)| ctx.collection.family_name(*id).map(str::to_string))
        .collect();
    (ctx, names)
}

pub fn s11() -> Row {
    let mut plain = build(s11_app, None, Net::Dummy);
    let a = shot(&mut plain, 0.0, "s11-font-unregistered");
    let (ctx, names) = fira_ctx();
    let mut reg = build(s11_app, Some(ctx), Net::Dummy);
    let b = shot(&mut reg, 0.0, "s11-font-registered");
    let widths = |s: &Shot| (ink_extent(s, 12, 48), ink_extent(s, 52, 88));
    let (ai, aw) = widths(&a);
    let (bi, bw) = widths(&b);
    let mono = |i: u32, w: u32| i * 10 >= w * 9;
    row(
        "S11",
        yes_no(mono(bi, bw) && !mono(ai, aw)),
        &[&a, &b],
        format!(
            "registered families {names:?}; ink width i-line/W-line: unregistered {ai}/{aw}, \
             registered {bi}/{bw}"
        ),
    )
}

// ---------------------------------------------------------------- S12
const S12_CSS: &str = "
body { margin: 0; padding: 10px; }
button { display: block; width: 160px; height: 40px; margin-bottom: 20px; border: 0;
         background-color: rgb(210,210,210); }
button:focus { background-color: rgb(255,220,0); }
button:focus-visible { box-shadow: 0 0 0 6px rgb(220,0,220); }
";

fn s12_app() -> Element {
    rsx! {
        style { {S12_CSS} }
        button { "first (Tab)" }
        button { "second (click)" }
    }
}

pub fn s12() -> Row {
    let mut doc = build(s12_app, None, Net::Dummy);
    let none = shot(&mut doc, 0.0, "s12-focus-none");
    key(&mut doc, Key::Tab, Code::Tab);
    let tab = shot(&mut doc, 0.0, "s12-focus-tab");
    click(&mut doc, 80.0, 90.0);
    let clicked = shot(&mut doc, 0.0, "s12-focus-click");
    let state = |s: &Shot, y: u32| format!("bg={} ring={}", hue(s.px(80, y)), hue(s.px(185, y)));
    let tab_ring = hue(tab.px(185, 30)) == "magenta";
    let click_ring = hue(clicked.px(185, 90)) == "magenta";
    let verdict = match (tab_ring, click_ring) {
        (true, false) => "YES",
        (true, true) => "PARTIAL (matches on pointer focus too)",
        _ => "NO",
    };
    row(
        "S12",
        verdict,
        &[&none, &tab, &clicked],
        format!(
            "before: b1 {}; after Tab: b1 {}; after click on b2: b1 {} / b2 {}",
            state(&none, 30),
            state(&tab, 30),
            state(&clicked, 30),
            state(&clicked, 90)
        ),
    )
}

// ---------------------------------------------------------------- S13
const S13_CSS: &str = "
body { margin: 0; padding: 10px; font-size: 20px; }
.t { width: 150px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
     background-color: rgb(235,235,235); margin-bottom: 20px; }
.f { width: 150px; white-space: nowrap; overflow: hidden; margin-bottom: 20px;
     mask-image: linear-gradient(to right, black 70%, transparent); }
.c { width: 150px; white-space: nowrap; overflow: hidden; }
";

fn s13_app() -> Element {
    rsx! {
        style { {S13_CSS} }
        div { class: "t", "The quick brown fox jumps over" }
        div { class: "f", "The quick brown fox jumps over" }
        div { class: "c", "The quick brown fox jumps over" }
        div { style: "font-size: 12px", "1: text-overflow: ellipsis  2: mask-image fade  3: plain clip" }
    }
}

pub fn s13() -> Row {
    let mut doc = build(s13_app, None, Net::Dummy);
    let s = shot(&mut doc, 0.0, "s13-ellipsis-and-fade");
    // Rightmost 12 px of each row: ellipsis dots are sparse; a fade is lighter than a clip.
    let darkness = |y0: u32| -> u32 {
        (148..160)
            .flat_map(|x| (y0..y0 + 24).map(move |y| (x, y)))
            .map(|(x, y)| 255 - u32::from(s.px(x, y)[0].min(s.px(x, y)[1])))
            .sum()
    };
    let (e, f, c) = (darkness(10), darkness(54), darkness(98));
    row(
        "S13",
        "see PNG",
        &[&s],
        format!(
            "ink in the last 12 px: ellipsis row={e} (bg included), fade row={f}, clip row={c}"
        ),
    )
}

// ---------------------------------------------------------------- S14
const S14_CSS: &str = "
body { margin: 0 }
.x { width: 120px; height: 60px; margin-bottom: 10px; }
.a { background-color: color-mix(in srgb, rgb(255,0,0) 50%, rgb(0,0,255)); }
.b { --accent: rgb(0,0,255); background-color: color-mix(in srgb, var(--accent) 20%, transparent); }
.c { background-color: color-mix(in oklab, rgb(255,0,0), rgb(0,0,255)); }
";

fn s14_app() -> Element {
    rsx! {
        style { {S14_CSS} }
        div { class: "x a", "srgb 50%" }
        div { class: "x b", "var 20% transparent" }
        div { class: "x c", "oklab" }
    }
}

pub fn s14() -> Row {
    let mut doc = build(s14_app, None, Net::Dummy);
    let s = shot(&mut doc, 0.0, "s14-color-mix");
    let (a, b, c) = (s.px(100, 50), s.px(100, 120), s.px(100, 190));
    let ok = (120..=136).contains(&a[0])
        && a[1] < 10
        && (120..=136).contains(&a[2])
        && b[2] > 240
        && b[0] > 180
        && b[0] < 225;
    row(
        "S14",
        yes_no(ok),
        &[&s],
        format!(
            "srgb 50% red/blue={a:?} (want ~128,0,128); var 20% over white={b:?} (want ~204,204,255); oklab={c:?}"
        ),
    )
}

// ---------------------------------------------------------------- S15 / S16
const S15_CSS: &str = "
body { margin: 0 }
.stripes { position: absolute; left: 0; top: 0; width: 400px; height: 300px;
  background-image: repeating-linear-gradient(90deg, rgb(0,0,0) 0px, rgb(0,0,0) 10px, rgb(255,255,255) 10px, rgb(255,255,255) 20px); }
.glass { position: absolute; left: 100px; top: 75px; width: 200px; height: 150px;
  backdrop-filter: blur(10px); background-color: rgba(255,255,255,0.1); }
";

fn s15_app() -> Element {
    rsx! {
        style { {S15_CSS} }
        div { class: "stripes" }
        div { class: "glass" }
    }
}

fn backend() -> &'static str {
    if cfg!(feature = "cpu-filters") {
        "cpu-filters"
    } else {
        "cpu"
    }
}

fn blurred(s: &Shot) -> (bool, String) {
    // x=105 sits in a black stripe (100..110); blurred it would be mid-grey.
    let inside = s.px(105, 150);
    let outside = s.px(5, 20);
    (
        (60..200).contains(&inside[0]),
        format!("inside glass {inside:?}, outside {outside:?}"),
    )
}

pub fn s15() -> Row {
    let mut doc = build(s15_app, None, Net::Dummy);
    let cpu = shot(&mut doc, 0.0, &format!("s15-backdrop-{}", backend()));
    let (cpu_ok, cpu_seen) = blurred(&cpu);
    let mut shots = vec![cpu];
    let hyb_seen = match harness::shot_hybrid(&mut doc, 0.0, "s15-backdrop-hybrid") {
        Ok(h) => {
            let (ok, seen) = blurred(&h);
            shots.push(h);
            format!("hybrid blurred={ok}: {seen}")
        }
        Err(e) => format!("hybrid: {e}"),
    };
    Row {
        id: "S15",
        verdict: yes_no(cpu_ok).to_string(),
        pngs: shots.iter().map(Shot::path).collect(),
        seen: format!("{} blurred={cpu_ok}: {cpu_seen}; {hyb_seen}", backend()),
    }
}

const S16_CSS: &str = "
body { margin: 0 }
.x { width: 180px; height: 100px; margin: 10px; background-color: rgb(220,0,0); color: white; }
.s { filter: saturate(0); }
.bl { filter: blur(4px); }
";

fn s16_app() -> Element {
    rsx! {
        style { {S16_CSS} }
        div { class: "x", "no filter" }
        div { class: "x s", "filter: saturate(0)" }
        div { class: "x bl", style: "position: absolute; left: 200px; top: 0px", "filter: blur(4px)" }
    }
}

fn filtered(s: &Shot) -> (bool, bool, String) {
    let sat = s.px(150, 180);
    let edge = s.px(211, 60); // 1 px inside the blurred box's left edge
    let sat_ok = hue(sat) == "grey";
    let blur_ok = edge[1] > 40; // unblurred edge is pure red (g=0)
    (
        sat_ok,
        blur_ok,
        format!("saturate(0) box={sat:?}, blur edge={edge:?}"),
    )
}

pub fn s16() -> Row {
    let mut doc = build(s16_app, None, Net::Dummy);
    let cpu = shot(&mut doc, 0.0, &format!("s16-filter-{}", backend()));
    let (cs, cb, cseen) = filtered(&cpu);
    let mut shots = vec![cpu];
    let (hyb_ok, hseen) = match harness::shot_hybrid(&mut doc, 0.0, "s16-filter-hybrid") {
        Ok(h) => {
            let (s, b, seen) = filtered(&h);
            shots.push(h);
            (s && b, format!("hybrid saturate={s} blur={b}: {seen}"))
        }
        Err(e) => (false, format!("hybrid: {e}")),
    };
    let verdict = match (cs && cb, hyb_ok) {
        (true, true) => "YES (both)",
        (false, true) => "PARTIAL (hybrid only)",
        (true, false) => "PARTIAL (cpu only)",
        _ => "NO",
    };
    Row {
        id: "S16",
        verdict: verdict.to_string(),
        pngs: shots.iter().map(Shot::path).collect(),
        seen: format!("{} saturate={cs} blur={cb}: {cseen}; {hseen}", backend()),
    }
}

pub const ALL: &[(&str, fn() -> Row)] = &[
    ("s1", s1),
    ("s2", s2),
    ("s3", s3),
    ("s4", s4),
    ("s5", s5),
    ("s6", s6),
    ("s7", s7),
    ("s8", s8),
    ("s9", s9),
    ("s10", s10),
    ("s11", s11),
    ("s12", s12),
    ("s13", s13),
    ("s14", s14),
    ("s15", s15),
    ("s16", s16),
];
