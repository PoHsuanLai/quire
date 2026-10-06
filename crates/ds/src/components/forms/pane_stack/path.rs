//! The path of a pane stack: the page it opens on and the pages pushed over it. A path is never
//! empty, so "the page shown" always exists.

/// Which way a stack moved to reach its path, `data-way`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, ds_core::word::Word)]
pub enum PaneWay {
    /// A page was pushed over the shown one: the new page comes in from the right.
    #[default]
    Push,
    /// The shown page was popped: the page under it comes back from the left.
    Pop,
}

/// The pages of a stack, root first. `K` names a page (an enum of the pane's pages, or an account
/// id); the page's content is the caller's, drawn from the key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PanePath<K> {
    root: K,
    pushed: Vec<K>,
}

impl<K: Clone> PanePath<K> {
    /// A path at its root page.
    pub fn new(root: K) -> Self {
        PanePath {
            root,
            pushed: Vec::new(),
        }
    }

    /// This path with `page` pushed on top.
    pub fn pushed(&self, page: K) -> Self {
        PanePath {
            root: self.root.clone(),
            pushed: [self.pushed.clone(), vec![page]].concat(),
        }
    }

    /// This path with its top page popped; a path at its root stays there.
    pub fn popped(&self) -> Self {
        PanePath {
            root: self.root.clone(),
            pushed: self.pushed[..self.pushed.len().saturating_sub(1)].to_vec(),
        }
    }

    /// The page shown.
    pub fn current(&self) -> &K {
        self.pushed.last().unwrap_or(&self.root)
    }

    /// The page under the one shown: what the back button names. `None` at the root.
    pub fn parent(&self) -> Option<&K> {
        match self.pushed.len() {
            0 => None,
            1 => Some(&self.root),
            n => self.pushed.get(n - 2),
        }
    }

    /// How many pages lie under the one shown: 0 at the root.
    pub fn depth(&self) -> usize {
        self.pushed.len()
    }

    /// Whether the page shown is the root.
    pub fn is_root(&self) -> bool {
        self.pushed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::PanePath;

    #[test]
    fn push_and_pop_walk_the_path() {
        let root = PanePath::new("accounts");
        let detail = root.pushed("dana");
        let deeper = detail.pushed("mail");
        let cases = [
            ("root", &root, "accounts", None, 0),
            ("detail", &detail, "dana", Some("accounts"), 1),
            ("deeper", &deeper, "mail", Some("dana"), 2),
        ];
        for (name, path, current, parent, depth) in cases {
            assert_eq!(*path.current(), current, "{name}");
            assert_eq!(path.parent().copied(), parent, "{name}");
            assert_eq!(path.depth(), depth, "{name}");
            assert_eq!(path.is_root(), depth == 0, "{name}");
        }
        assert_eq!(deeper.popped(), detail);
        assert_eq!(detail.popped(), root);
        assert_eq!(root.popped(), root, "the root cannot be popped");
    }
}
