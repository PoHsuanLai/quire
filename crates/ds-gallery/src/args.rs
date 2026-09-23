//! The gallery's command line: `ds-gallery [--page PAGE] [--snapshot DIR]`.
#![allow(unused_variables, dead_code)] // Freeze stubs: remove with the last todo!().

use crate::page::Page;
use std::path::PathBuf;

/// What the gallery was asked to do.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Args {
    /// Open on this page (the tokens page when absent).
    pub page: Option<Page>,
    /// Render every page and state into this directory as a contact sheet, then exit.
    pub snapshot: Option<PathBuf>,
}

/// A command line that is not one the gallery understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgsError(pub String);

/// Parse the arguments after the program name.
pub fn parse(args: impl Iterator<Item = String>) -> Result<Args, ArgsError> {
    todo!()
}
