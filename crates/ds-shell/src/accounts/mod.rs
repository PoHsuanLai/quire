//! The account sheets' views (design/31 section 4.5 and 5): the consent alert, the add-account
//! steps (provider list, sign-in form, browser wait, device code, review) and the small parts apps
//! show (account picker, no-account state, badge, limited note). Every component takes plain props
//! and reports events; nothing here keeps a value the host owns, and no porter type is named: a
//! host maps its sheet view to these props. A secret typed into a field travels as [`Hidden`] and
//! prints as `<hidden>`.

pub(crate) mod adapter;
pub(crate) mod badge;
pub(crate) mod browser_wait;
pub(crate) mod consent;
pub(crate) mod failed;
pub(crate) mod frame;
pub mod hidden;
pub(crate) mod limited;
pub mod model;
pub(crate) mod no_account;
pub(crate) mod picker;
pub(crate) mod providers;
pub(crate) mod review;
pub(crate) mod sheet;
pub(crate) mod show_code;
pub(crate) mod sign_in;
pub(crate) mod waiting;
pub(crate) mod wording;
pub(crate) mod working;
