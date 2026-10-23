//! The cascade layers of a document (ARCHITECTURE.md section 10). The design-system stylesheet
//! is `@layer ds`; a consumer's own sheets are `@layer app`; the chosen look is `@layer look`;
//! the person's own stylesheets are unlayered. An unlayered rule beats every layered one whatever
//! its specificity, and a later layer beats an earlier one, so the order is the person's, then
//! the look, then app, then ds.

/// The layer every design-system rule is in.
pub const DS: &str = "ds";
/// The layer a consumer's own stylesheets are in: after `ds`, before the person's rules.
pub const APP: &str = "app";
/// The layer the chosen look is in: after `app` (a look restyles every app the same way), before
/// the person's rules.
pub const LOOK: &str = "look";
/// The statement that fixes the layer order. Every sheet that opens a layer starts with it, so
/// the order holds whichever sheet the renderer parses first.
pub const ORDER: &str = "@layer ds, app, look;";

/// `css` as a consumer's sheet: in the `app` layer, after the order statement.
pub fn app(css: &str) -> String {
    format!("{ORDER}\n@layer {APP} {{\n{}\n}}\n", css.trim_end())
}

/// `css` as the chosen look: in the `look` layer, after the order statement.
pub fn look(css: &str) -> String {
    format!("{ORDER}\n@layer {LOOK} {{\n{}\n}}\n", css.trim_end())
}
