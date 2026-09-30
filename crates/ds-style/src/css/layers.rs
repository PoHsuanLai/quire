//! The cascade layers of a document (ARCHITECTURE.md section 10). The design-system stylesheet
//! is `@layer ds`; a consumer's own sheets are `@layer app`; the person's `style.css` is
//! unlayered. An unlayered rule beats every layered one whatever its specificity, and a later
//! layer beats an earlier one, so the order is user, then app, then ds.

/// The layer every design-system rule is in.
pub const DS: &str = "ds";
/// The layer a consumer's own stylesheets are in: after `ds`, before the person's rules.
pub const APP: &str = "app";
/// The statement that fixes the layer order. Every sheet that opens a layer starts with it, so
/// the order holds whichever sheet the renderer parses first.
pub const ORDER: &str = "@layer ds, app;";

/// `css` as a consumer's sheet: in the `app` layer, after the order statement.
pub fn app(css: &str) -> String {
    format!("{ORDER}\n@layer {APP} {{\n{}\n}}\n", css.trim_end())
}
