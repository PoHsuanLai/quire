//! `CamelCase` to `snake_case`, matching serde's `rename_all = "snake_case"` for the plain
//! identifiers this workspace's settings enums use (no digits, no acronyms).

/// `ident` (a variant or field name as written in Rust) as serde would rename it.
pub(crate) fn to_snake_case(ident: &str) -> String {
    let mut out = String::new();
    for (index, ch) in ident.chars().enumerate() {
        if ch.is_uppercase() {
            if index != 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::to_snake_case;

    #[test]
    fn matches_serdes_snake_case_rename() {
        const CASES: &[(&str, &str)] = &[
            ("System", "system"),
            ("ForceWhite", "force_white"),
            ("SameAsLight", "same_as_light"),
            ("Postmark", "postmark"),
            ("On", "on"),
        ];
        for (given, want) in CASES {
            assert_eq!(to_snake_case(given), *want, "{given}");
        }
    }
}
