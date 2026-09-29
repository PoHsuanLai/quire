//! The top layer: what wires every layer together. The stylesheet in its cascade order, every
//! component sheet registered in one ordered list, and the `Ds` root that draws the overlay and
//! toast hosts and the window frame around a surface.

pub(crate) mod ds;
pub(crate) mod sheets;
pub(crate) mod stylesheet;

#[cfg(test)]
mod stored_words;
