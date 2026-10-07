//! Reading a Blitz document into a [`Scene`]: every element that paints an edge, and the text
//! and glyphs inside it.

use crate::inset::bounds::Bounds;
use crate::inset::ink;
use crate::inset::paint::{Ground, painted};
use crate::inset::scene::{Content, ContentKind, Scene, VisibleBox};
use blitz_dom::{BaseDocument, LocalName, Node, NodeId};
use rustc_hash::FxHashMap;

/// Below this opacity an element and what is in it are not seen.
const SEEN: f32 = 0.05;

/// Read `doc` into a scene.
pub fn read(doc: &BaseDocument) -> Scene {
    let mut walk = Walk {
        doc,
        scene: Scene::default(),
        nodes: Vec::new(),
        owner: FxHashMap::default(),
        layouts: Vec::new(),
    };
    let root = doc.root_element().id;
    walk.visit(
        root,
        Around {
            ground: Ground::page(),
            opacity: 1.0,
            owner: None,
            placed: false,
            path: String::new(),
        },
    );
    walk.texts();
    walk.scene
}

/// What an element inherits from the elements around it.
#[derive(Clone)]
struct Around {
    ground: Ground,
    opacity: f32,
    /// The nearest box around, by index into the scene.
    owner: Option<usize>,
    /// Whether an absolutely positioned element between it and `owner` places it by its own
    /// offsets: a corner mark, not content to inset.
    placed: bool,
    path: String,
}

struct Walk<'a> {
    doc: &'a BaseDocument,
    scene: Scene,
    /// The element of each box, in step with `scene.boxes`.
    nodes: Vec<NodeId>,
    /// The box around each element visited.
    owner: FxHashMap<NodeId, (Option<usize>, bool)>,
    /// Every inline root and text input with something to ink, and the box around it.
    layouts: Vec<(NodeId, (Option<usize>, bool))>,
}

impl Walk<'_> {
    fn visit(&mut self, id: NodeId, around: Around) {
        let Some(node) = self.doc.get_node(id) else {
            return;
        };
        let Some(element) = node.element_data() else {
            return;
        };
        let Some(styles) = node.primary_styles() else {
            return;
        };
        let opacity = around.opacity * styles.get_effects().opacity;
        if styles.clone_display().is_none() || opacity < SEEN {
            return;
        }
        let shown = format!("{:?}", styles.get_inherited_box().visibility) == "Visible";
        let path = join(&around.path, &label(node));
        let tag = element.name.local.as_ref();
        let positioned = matches!(
            format!("{:?}", styles.get_box().position)
                .to_ascii_lowercase()
                .as_str(),
            "absolute" | "fixed"
        );
        let mut inner = Around {
            opacity,
            path: path.clone(),
            placed: around.placed || positioned,
            ..around.clone()
        };
        // A gradient mask is a fade (`.ds-truncate`); a url mask is an icon's shape.
        let masked = styles
            .get_svg()
            .mask_image
            .0
            .iter()
            .any(|layer| format!("{layer:?}").to_ascii_lowercase().contains("url"));
        let pictured = styles
            .get_background()
            .background_image
            .0
            .iter()
            .any(|layer| format!("{layer:?}").to_ascii_lowercase().contains("url"));
        let drawn = matches!(tag, "svg" | "img" | "canvas" | "video")
            || masked
            || (pictured && node.children.is_empty());
        if shown && drawn {
            let owner = if inner.placed { None } else { around.owner };
            self.glyph(id, node, owner, &path);
            return;
        }
        let own = self.doc.inline_fragment_rects(id).is_none();
        if shown && own {
            inner = self.boxed(id, node, inner);
        }
        self.owner.insert(id, (inner.owner, inner.placed));
        let has_text = node.flags.is_inline_root() && element.inline_layout_data.is_some()
            || element.text_input_data().is_some();
        if shown && has_text {
            self.layouts.push((id, (inner.owner, inner.placed)));
        }
        for &child in node.children.iter() {
            self.visit(child, inner.clone());
        }
    }

    /// `around` with the box `node` makes, if it paints an edge.
    fn boxed(&mut self, id: NodeId, node: &Node, around: Around) -> Around {
        let Some(found) = painted(node, &around.ground) else {
            return around;
        };
        let Some(rect) = self.doc.get_client_bounding_rect(id) else {
            return around;
        };
        let bounds = Bounds::at(
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        );
        let ground = found.ground;
        let edged = (found.edges.left || found.edges.right)
            && bounds.width() > 0.0
            && bounds.height() > 0.0;
        if !edged {
            return Around { ground, ..around };
        }
        self.scene.boxes.push(VisibleBox {
            path: around.path.clone(),
            tokens: tokens(self.doc, node),
            bounds,
            edges: found.edges,
            paints: found.paints,
        });
        self.nodes.push(id);
        Around {
            ground,
            owner: Some(self.scene.boxes.len() - 1),
            placed: false,
            ..around
        }
    }

    fn glyph(&mut self, id: NodeId, node: &Node, owner: Option<usize>, path: &str) {
        let (Some(owner), Some(rect)) = (owner, self.doc.get_client_bounding_rect(id)) else {
            return;
        };
        let _ = node;
        self.scene.contents.push(Content {
            bounds: Bounds::at(
                rect.x as f32,
                rect.y as f32,
                rect.width as f32,
                rect.height as f32,
            ),
            kind: ContentKind::Glyph,
            label: self.below(owner, id, path),
            owner,
        });
    }

    /// The text runs of every layout found, each owned by the box around the element whose
    /// style it carries.
    fn texts(&mut self) {
        for (id, owner) in std::mem::take(&mut self.layouts) {
            let Some(node) = self.doc.get_node(id) else {
                continue;
            };
            let Some(element) = node.element_data() else {
                continue;
            };
            let scroll = self.doc.viewport_scroll();
            let at = node.unrounded_absolute_position(0.0, 0.0);
            let layout = node.unrounded_layout();
            let x =
                f64::from(at.x) + f64::from(layout.border.left + layout.padding.left) - scroll.x;
            let y = f64::from(at.y) + f64::from(layout.border.top + layout.padding.top) - scroll.y;
            let runs = match (&element.inline_layout_data, element.text_input_data()) {
                (Some(inline), _) => ink::runs(&inline.layout, &inline.text, x, y),
                (None, Some(input)) => match input.editor.try_layout() {
                    Some(layout) => ink::runs(layout, input.editor.raw_text(), x, y),
                    None => continue,
                },
                (None, None) => continue,
            };
            for run in runs {
                let (owner, placed) = self.owner.get(&run.element).copied().unwrap_or(owner);
                let (false, Some(owner)) = (placed, owner) else {
                    continue;
                };
                let path = self.path_of(run.element);
                self.scene.contents.push(Content {
                    bounds: run.bounds,
                    kind: ContentKind::Text,
                    label: format!("{} \"{}\"", self.below(owner, run.element, &path), run.text),
                    owner,
                });
            }
        }
    }

    /// The path of `element` from the document root, as the walk labelled it.
    fn path_of(&self, element: NodeId) -> String {
        let mut chain = Vec::new();
        let mut at = Some(element);
        while let Some(node) = at.and_then(|id| self.doc.get_node(id)) {
            if node.is_element() {
                chain.push(label(node));
            }
            at = node.parent;
        }
        chain.reverse();
        chain.join(" > ")
    }

    /// The path of `element` below the box `owner`: what to look for inside it.
    fn below(&self, owner: usize, element: NodeId, path: &str) -> String {
        let boxed = &self.scene.boxes[owner].path;
        let _ = self.nodes[owner];
        match path.strip_prefix(boxed.as_str()) {
            Some("") => "(the box itself)".to_owned(),
            Some(rest) => rest.trim_start_matches(" > ").to_owned(),
            None => format!("{element:?}"),
        }
    }
}

/// What an [`Allow`] can name on `node`: its classes and marks, and its parent's marks with a
/// `^` before them (a field's frame has no size of its own; the field around it does).
fn tokens(doc: &BaseDocument, node: &Node) -> Vec<String> {
    let parent = node.parent.and_then(|parent| doc.get_node(parent));
    let above = parent.map(marks).unwrap_or_default();
    classes(node)
        .into_iter()
        .chain(marks(node))
        .chain(above.into_iter().map(|mark| format!("^{mark}")))
        .collect()
}

fn classes(node: &Node) -> Vec<String> {
    node.attr(LocalName::from("class"))
        .map(|classes| {
            classes
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The short `data-*` attributes of `node` as `[data-size=mini]`: what an [`Allow`] names beside
/// a class, and what tells two sizes of one component apart in a path.
fn marks(node: &Node) -> Vec<String> {
    let Some(attributes) = node.attrs() else {
        return Vec::new();
    };
    attributes
        .iter()
        .filter(|attribute| attribute.value.len() <= 12 && !attribute.value.contains(' '))
        .filter(|attribute| attribute.name.local.as_ref().starts_with("data-"))
        .filter(|attribute| attribute.name.local.as_ref() != "data-dioxus-id")
        .map(|attribute| format!("[{}={}]", attribute.name.local, attribute.value))
        .collect()
}

/// `tag.class.class#id`.
fn label(node: &Node) -> String {
    let tag = node
        .element_data()
        .map_or("", |element| element.name.local.as_ref());
    let mut text = tag.to_owned();
    for class in classes(node) {
        text.push('.');
        text.push_str(&class);
    }
    for mark in marks(node) {
        text.push_str(&mark);
    }
    if let Some(id) = node.attr(LocalName::from("id")) {
        text.push('#');
        text.push_str(id);
    }
    text
}

fn join(parent: &str, label: &str) -> String {
    match parent.is_empty() {
        true => label.to_owned(),
        false => format!("{parent} > {label}"),
    }
}
