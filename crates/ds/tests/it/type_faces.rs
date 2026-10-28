//! The faces the stylesheet asks for, and the Editorial look Extension that names mail's faces in
//! their place: every property it sets is one the `.ds` block declares, so it restyles and never
//! invents a token.

use ds::stylesheet;
use ds_style::tokens::type_scale::Family;

/// The Editorial look's sheet, as quire ships it.
const EDITORIAL: &str = include_str!("../../../../extensions/org.quire.look.Editorial/look.css");

/// The `.ds` token block of the stylesheet.
fn ds_block(css: &str) -> &str {
    css.lines()
        .filter_map(|line| line.strip_prefix(".ds{"))
        .find(|body| body.contains("--font-ui:"))
        .unwrap_or_else(|| panic!("no .ds token block"))
}

/// The custom properties the Editorial look sets, with their values.
fn editorial() -> Vec<(&'static str, &'static str)> {
    EDITORIAL
        .lines()
        .filter_map(|line| {
            let (name, value) = line.trim().strip_prefix("--")?.split_once(':')?;
            Some((name, value.trim().trim_end_matches(';')))
        })
        .collect()
}

#[test]
fn the_editorial_look_sets_only_declared_tokens_and_names_every_interface_face() {
    let declared = ds_block(stylesheet());
    let set = editorial();
    for (name, value) in &set {
        assert!(
            declared.contains(&format!("--{name}:")),
            "--{name} is not a .ds token"
        );
        assert!(
            !declared.contains(&format!("--{name}:{value};")),
            "--{name} repeats the standard value"
        );
    }
    for family in [Family::Display, Family::Ui, Family::Data] {
        let name = family.var();
        let stack = set
            .iter()
            .find(|(set, _)| format!("--{set}") == name.as_str())
            .map(|(_, stack)| *stack)
            .unwrap_or_else(|| panic!("the Editorial look leaves {} alone", name.as_str()));
        let faces: Vec<&str> = stack
            .split(',')
            .map(|face| face.trim().trim_matches('"'))
            .collect();
        assert_eq!(
            faces.get(1),
            Some(&"Inter"),
            "{stack}: a glyph the face lacks comes from Inter"
        );
    }
}
