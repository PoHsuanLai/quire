//! What a field's own Rust type says about its [`ds_settings::schema::KeyKind`]
//! (design/22-SETTINGS.md section 9.1): a `range` attribute means `Bounded`; failing that, the
//! type's own name decides `Text`, `Colour` or `List`; anything else is assumed to be a closed
//! enum and resolved at run time through `SchemaVariants` (`crate::gen_struct`).

/// Which branch of `KeyKind` a field's type picks, syntactically — no trait resolution, since
/// a proc-macro only ever sees tokens.
pub(crate) enum Shape {
    /// `#[settings(range = "...")]` was given: a bounded number, whatever the newtype.
    Bounded,
    /// The field's type is exactly `String`.
    Text,
    /// The field's type is exactly `Hex` (any path ending in that segment).
    Colour,
    /// The field's type is `Vec<T>`: a list of `T`'s own kind. Boxed: `syn::Type` is large
    /// enough on its own to blow up every other, data-less variant's size (`clippy::large_enum_variant`).
    List(Box<syn::Type>),
    /// Anything else: a closed enum, resolved through `SchemaVariants` at run time.
    EnumType,
}

/// The [`Shape`] of `ty`, given whether a `range` attribute was present.
pub(crate) fn shape_of(ty: &syn::Type, has_range: bool) -> Shape {
    if has_range {
        return Shape::Bounded;
    }
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        if segment.ident == "String" {
            return Shape::Text;
        }
        if segment.ident == "Hex" {
            return Shape::Colour;
        }
        if segment.ident == "Vec"
            && let syn::PathArguments::AngleBracketed(generics) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner)) = generics.args.first()
        {
            return Shape::List(Box::new(inner.clone()));
        }
    }
    Shape::EnumType
}

#[cfg(test)]
mod tests {
    use super::{Shape, shape_of};
    use quote::ToTokens;

    fn ty(text: &str) -> syn::Type {
        syn::parse_str(text).unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    #[test]
    fn range_always_wins() {
        assert!(matches!(shape_of(&ty("String"), true), Shape::Bounded));
    }

    #[test]
    fn string_is_text() {
        assert!(matches!(shape_of(&ty("String"), false), Shape::Text));
    }

    #[test]
    fn hex_is_colour() {
        assert!(matches!(shape_of(&ty("Hex"), false), Shape::Colour));
        assert!(matches!(shape_of(&ty("tokens::Hex"), false), Shape::Colour));
    }

    #[test]
    fn vec_is_a_list_of_its_element() {
        match shape_of(&ty("Vec<Accent>"), false) {
            Shape::List(inner) => assert_eq!(inner.to_token_stream().to_string(), "Accent"),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn anything_else_is_an_enum_type() {
        assert!(matches!(shape_of(&ty("Theme"), false), Shape::EnumType));
        assert!(matches!(shape_of(&ty("Px"), false), Shape::EnumType));
    }
}
