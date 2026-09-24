//! What a field's own Rust type says about its [`ds_settings::schema::KeyKind`]
//! (design/22-SETTINGS.md section 9.1): a `range` attribute means `Bounded`; `text` means
//! `Text`; failing both, the type's own name decides `Text`, `Colour` or `List`, a known
//! numeric type without a range is an error, and anything else is assumed to be a closed enum
//! resolved at run time through `SchemaVariants` (`crate::gen_struct`).

use crate::attrs::KindHint;
use std::fmt;

/// Which branch of `KeyKind` a field's type picks, syntactically — no trait resolution, since
/// a proc-macro only ever sees tokens.
pub(crate) enum Shape {
    /// `#[settings(range = "min..=max")]` was given: a bounded number, whatever the newtype.
    Bounded {
        /// The low end.
        min: i64,
        /// The high end.
        max: i64,
    },
    /// The field's type is `String`, `PathBuf` or `Cow<str>` (any path ending in one), or
    /// `#[settings(text)]` said so.
    Text,
    /// The field's type is exactly `Hex` (any path ending in that segment).
    Colour,
    /// The field's type is `Vec<T>`: a list of `T`'s own kind. Boxed: `syn::Type` is large
    /// enough on its own to blow up every other, data-less variant's size (`clippy::large_enum_variant`).
    List(Box<syn::Type>),
    /// Anything else: a closed enum, resolved through `SchemaVariants` at run time.
    EnumType,
}

/// Why a field's type cannot be given a shape.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ShapeError {
    /// A number (a primitive or one of ds-settings' unit newtypes) with no `range`: a slider
    /// needs both ends.
    MissingRange {
        /// The field's name.
        field: String,
    },
}

impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapeError::MissingRange { field } => write!(
                f,
                "MissingRange {{ field: {field} }}: `{field}` is a number, so it needs \
                 #[settings(range = \"min..=max\")] to be drawn as a slider"
            ),
        }
    }
}

/// The numeric types a field is recognised by: the primitives and ds-settings' own units
/// (`crates/ds-settings/src/units.rs`). A caller's own numeric newtype is caught at compile
/// time instead, by `SchemaVariants`' `on_unimplemented` message.
const NUMBERS: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "Px", "Ms", "Percent", "Count", "Fraction", "Scalar", "Units",
];

/// The [`Shape`] of `field`'s type `ty`, given what its attributes asked for.
pub(crate) fn shape_of(field: &str, ty: &syn::Type, hint: KindHint) -> Result<Shape, ShapeError> {
    match hint {
        KindHint::Bounded { min, max } => return Ok(Shape::Bounded { min, max }),
        KindHint::Text => return Ok(Shape::Text),
        KindHint::Infer => {}
    }
    let syn::Type::Path(type_path) = ty else {
        return Ok(Shape::EnumType);
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Ok(Shape::EnumType);
    };
    let name = segment.ident.to_string();
    Ok(match name.as_str() {
        "String" | "PathBuf" => Shape::Text,
        "Cow" if borrows_str(&segment.arguments) => Shape::Text,
        "Hex" => Shape::Colour,
        "Vec" => match first_type(&segment.arguments) {
            Some(inner) => Shape::List(Box::new(inner.clone())),
            None => Shape::EnumType,
        },
        number if NUMBERS.contains(&number) => {
            return Err(ShapeError::MissingRange {
                field: field.to_owned(),
            });
        }
        _ => Shape::EnumType,
    })
}

/// The first type argument of `Name<T, ...>`.
fn first_type(arguments: &syn::PathArguments) -> Option<&syn::Type> {
    let syn::PathArguments::AngleBracketed(generics) = arguments else {
        return None;
    };
    generics.args.iter().find_map(|arg| match arg {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    })
}

/// Whether `Cow<'_, str>`'s type argument is `str`.
fn borrows_str(arguments: &syn::PathArguments) -> bool {
    matches!(
        first_type(arguments),
        Some(syn::Type::Path(path)) if path.path.is_ident("str")
    )
}

#[cfg(test)]
mod tests {
    use super::{Shape, ShapeError, shape_of};
    use crate::attrs::KindHint;
    use quote::ToTokens;

    fn ty(text: &str) -> syn::Type {
        syn::parse_str(text).unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    fn infer(text: &str) -> Result<Shape, ShapeError> {
        shape_of("f", &ty(text), KindHint::Infer)
    }

    #[test]
    fn range_always_wins() {
        assert!(matches!(
            shape_of("f", &ty("String"), KindHint::Bounded { min: 0, max: 1 }),
            Ok(Shape::Bounded { min: 0, max: 1 })
        ));
    }

    #[test]
    fn text_is_recognised_by_type() {
        const CASES: &[&str] = &[
            "String",
            "std::string::String",
            "PathBuf",
            "std::path::PathBuf",
            "Cow<'static, str>",
            "std::borrow::Cow<'a, str>",
        ];
        for text in CASES {
            assert!(matches!(infer(text), Ok(Shape::Text)), "{text}");
        }
        assert!(
            matches!(infer("Cow<'static, [u8]>"), Ok(Shape::EnumType)),
            "Cow of bytes is not text"
        );
    }

    #[test]
    fn the_text_attribute_makes_a_newtype_text() {
        assert!(matches!(
            shape_of("theme", &ty("SoundTheme"), KindHint::Text),
            Ok(Shape::Text)
        ));
    }

    #[test]
    fn hex_is_colour() {
        assert!(matches!(infer("Hex"), Ok(Shape::Colour)));
        assert!(matches!(infer("tokens::Hex"), Ok(Shape::Colour)));
    }

    #[test]
    fn vec_is_a_list_of_its_element() {
        match infer("Vec<Accent>") {
            Ok(Shape::List(inner)) => assert_eq!(inner.to_token_stream().to_string(), "Accent"),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn a_number_without_a_range_is_missing_range() {
        const CASES: &[&str] = &["u8", "i64", "f32", "Px", "ds_settings::Ms", "Percent"];
        for text in CASES {
            assert_eq!(
                shape_of("size", &ty(text), KindHint::Infer).err(),
                Some(ShapeError::MissingRange {
                    field: "size".to_owned()
                }),
                "{text}"
            );
        }
        let message = ShapeError::MissingRange {
            field: "size".to_owned(),
        }
        .to_string();
        assert!(
            message.starts_with("MissingRange { field: size }"),
            "{message}"
        );
        assert!(message.contains("range = "), "{message}");
    }

    #[test]
    fn anything_else_is_an_enum_type() {
        assert!(matches!(infer("Theme"), Ok(Shape::EnumType)));
        assert!(matches!(infer("(u8, u8)"), Ok(Shape::EnumType)));
    }
}
