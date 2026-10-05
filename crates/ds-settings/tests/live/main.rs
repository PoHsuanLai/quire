//! `org.quire.SettingsModule1` end to end: the skeleton and the proxy on a private bus, the role
//! hook, the never-settable rule over live schemas, and the schema JSON.
#![cfg(feature = "live")]

mod bus;
mod module;
mod round_trip;
mod schema;
