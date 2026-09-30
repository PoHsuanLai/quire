//! `AppStyle`: a consumer's own stylesheet, in the `app` cascade layer (ARCHITECTURE.md section
//! 10). The design system is `@layer ds`, a consumer's sheets come after it in `@layer app`, and
//! the person's `style.css` is unlayered, so the order of who wins is user, then app, then ds
//! whatever the specificity of each rule. The text starts with the layer order statement, so the
//! order holds whichever sheet the renderer parses first.

use dioxus::prelude::*;
use ds_style::css::layers;
use std::borrow::Cow;

/// A `<style>` holding `css` in the `app` layer. Draw it where the sheet used to be written
/// (`style { {CSS} }`); the sheet must not open `@layer ds` (`ds_lint::Rule::LayerDs`).
#[component]
pub fn AppStyle(#[props(into)] css: Cow<'static, str>) -> Element {
    let text = use_memo(use_reactive!(|css| layers::app(&css)));
    rsx! {
        style { {text.read().clone()} }
    }
}
