//! Values on the wire. A live key's value is a [`toml::Value`] in Rust (as `KeySpec::default`
//! is) and one of four D-Bus types on the bus:
//!
//! | `toml::Value` | D-Bus |
//! | --- | --- |
//! | `Boolean` | `b` |
//! | `Integer` | `x` |
//! | `String` | `s` |
//! | `Array` of `String` | `as` |
//!
//! Anything else (floats, dates, tables, mixed arrays) has no wire form and is
//! [`LiveError::BadValue`].

use zbus::zvariant::Value;

use super::LiveError;

/// The variant `v` carrying `value`.
pub fn to_wire(value: &toml::Value) -> Result<Value<'static>, LiveError> {
    match value {
        toml::Value::Boolean(on) => Ok(Value::Bool(*on)),
        toml::Value::Integer(n) => Ok(Value::I64(*n)),
        toml::Value::String(text) => Ok(Value::from(text.clone())),
        toml::Value::Array(items) => items
            .iter()
            .map(|item| match item {
                toml::Value::String(text) => Ok(text.clone()),
                other => Err(unsupported(other)),
            })
            .collect::<Result<Vec<String>, _>>()
            .map(Value::from),
        other => Err(unsupported(other)),
    }
}

/// The value a received variant `v` carries.
pub fn from_wire(value: &Value<'_>) -> Result<toml::Value, LiveError> {
    match value {
        Value::Bool(on) => Ok(toml::Value::Boolean(*on)),
        Value::I64(n) => Ok(toml::Value::Integer(*n)),
        Value::Str(text) => Ok(toml::Value::String(text.to_string())),
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::Str(text) => Ok(toml::Value::String(text.to_string())),
                other => Err(LiveError::BadValue(format!(
                    "a list holds strings, got signature {}",
                    other.value_signature()
                ))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(toml::Value::Array),
        Value::Value(inner) => from_wire(inner),
        other => Err(LiveError::BadValue(format!(
            "no live value has signature {}",
            other.value_signature()
        ))),
    }
}

fn unsupported(value: &toml::Value) -> LiveError {
    LiveError::BadValue(format!("a {} has no wire form", value.type_str()))
}

#[cfg(test)]
mod tests {
    use super::{from_wire, to_wire};

    fn cases() -> Vec<toml::Value> {
        vec![
            toml::Value::Boolean(true),
            toml::Value::Boolean(false),
            toml::Value::Integer(-7),
            toml::Value::Integer(i64::MAX),
            toml::Value::String(String::new()),
            toml::Value::String("héllo".to_owned()),
            toml::Value::Array(Vec::new()),
            toml::Value::Array(vec!["a".into(), "b".into()]),
        ]
    }

    #[test]
    fn every_wire_type_round_trips() {
        for value in cases() {
            let wire = to_wire(&value).expect("has a wire form");
            assert_eq!(from_wire(&wire).expect("reads back"), value, "{value:?}");
        }
    }

    #[test]
    fn a_value_without_a_wire_form_is_bad_value() {
        for value in [
            toml::Value::Float(1.5),
            toml::Value::Table(toml::Table::new()),
            toml::Value::Array(vec![toml::Value::Integer(1)]),
        ] {
            assert!(to_wire(&value).is_err(), "{value:?}");
        }
    }

    #[test]
    fn a_variant_without_a_live_type_is_bad_value() {
        assert!(from_wire(&zbus::zvariant::Value::F64(1.0)).is_err());
        assert!(from_wire(&zbus::zvariant::Value::from(vec![1_i64])).is_err());
    }

    #[test]
    fn a_nested_variant_is_unwrapped() {
        let nested = zbus::zvariant::Value::Value(Box::new(zbus::zvariant::Value::I64(3)));
        assert_eq!(from_wire(&nested).unwrap(), toml::Value::Integer(3));
    }
}
