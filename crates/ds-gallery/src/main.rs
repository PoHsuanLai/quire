//! The quire gallery: every component across theme, accent, motion, material, blur and Space
//! preset; a matrix page; a motion lab; and `--snapshot DIR` for a contact sheet.

mod args;
mod page;

use dioxus as _;
use ds as _;
use ds_native as _;
use ds_settings as _;

fn main() {
    let args = args::parse(std::env::args().skip(1));
    todo!("run the gallery with {args:?}")
}
