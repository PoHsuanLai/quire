//! Which tag each live frame document carries, per quire document. A frame's document is parsed,
//! and asks for its first images, before Blitz attaches it to its `iframe`, so nothing can read
//! the element's `data-frame-tag` while those first requests are made. The book holds a frame's
//! requests for the app until the document it came from is found under its `iframe` (the next
//! frame's walk, `crate::frame_tree`), then puts them to the app with the tag. `data:` never
//! waits: it is never put to the app.
//!
//! The lookups (`crate::frames::{tag_of, frame_by_tag}`) reach every book on the calling thread,
//! so they work from an app's handler in the window and in a test alike without a context.

use crate::frame_tag::FrameTag;
use crate::origin::FrameId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

/// A frame's request for the app, run with the frame's tag once it is known.
pub(crate) type Held = Box<dyn FnOnce(Option<FrameTag>) + Send>;

/// One quire document's frames.
#[derive(Default)]
struct Book {
    /// Every frame found under an `iframe`, and its tag if the element has one.
    tags: HashMap<FrameId, Option<FrameTag>>,
    /// Requests from frames not found yet.
    held: HashMap<FrameId, Vec<Held>>,
}

/// One quire document's frames, shared by its frame net provider and its walk.
#[derive(Clone)]
pub(crate) struct FrameBook(Arc<Mutex<Book>>);

thread_local! {
    /// Every book of a document on this thread: the window's UI thread, or a test's.
    static BOOKS: RefCell<Vec<Weak<Mutex<Book>>>> = const { RefCell::new(Vec::new()) };
}

impl FrameBook {
    /// An empty book, reachable from this thread's lookups for as long as it lives.
    pub(crate) fn new() -> Self {
        let book = Arc::new(Mutex::new(Book::default()));
        BOOKS.with_borrow_mut(|books| {
            books.retain(|weak| weak.strong_count() > 0);
            books.push(Arc::downgrade(&book));
        });
        FrameBook(book)
    }

    /// Run `request` with `frame`'s tag now if the frame has been found, or when it is.
    pub(crate) fn ask(&self, frame: FrameId, request: Held) {
        let mut book = lock(&self.0);
        match book.tags.get(&frame).cloned() {
            Some(tag) => {
                drop(book);
                request(tag);
            }
            None => book.held.entry(frame).or_default().push(request),
        }
    }

    /// `frame`'s tag, if it has been found and its `iframe` has one.
    pub(crate) fn tag(&self, frame: FrameId) -> Option<FrameTag> {
        lock(&self.0).tags.get(&frame).cloned().flatten()
    }

    /// Record `live`, every frame now under an `iframe`: forget the frames gone (and whatever
    /// they still held), and put each newly found frame's held requests to the app with its tag.
    pub(crate) fn bind(&self, live: Vec<(FrameId, Option<FrameTag>)>) {
        let released = {
            let mut guard = lock(&self.0);
            let book = &mut *guard;
            let released = release(book, &live);
            book.tags = live.into_iter().collect();
            let tags = &book.tags;
            book.held.retain(|frame, _| tags.contains_key(frame));
            released
        };
        // Unlocked: the app's `decide` may look a tag up.
        for (tag, requests) in released {
            requests
                .into_iter()
                .for_each(|request| request(tag.clone()));
        }
    }
}

/// The held requests of each frame in `live` that `book` had not found yet, with its tag.
fn release(
    book: &mut Book,
    live: &[(FrameId, Option<FrameTag>)],
) -> Vec<(Option<FrameTag>, Vec<Held>)> {
    live.iter()
        .filter(|(frame, _)| !book.tags.contains_key(frame))
        .filter_map(|(frame, tag)| Some((tag.clone(), book.held.remove(frame)?)))
        .collect()
}

fn lock(book: &Mutex<Book>) -> MutexGuard<'_, Book> {
    book.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Every live book on this thread.
fn books() -> Vec<Arc<Mutex<Book>>> {
    BOOKS.with_borrow(|books| books.iter().filter_map(Weak::upgrade).collect())
}

/// The tag of `frame`'s `iframe`, if the frame is live in a document on this thread and its
/// element has one.
pub(crate) fn tag_of(frame: FrameId) -> Option<FrameTag> {
    books()
        .iter()
        .find_map(|book| lock(book).tags.get(&frame).cloned())
        .flatten()
}

/// The live frame whose `iframe` carries `tag`; the newest document if several do (an app gives
/// each frame its own tag, and a reloaded frame is a new document).
pub(crate) fn frame_by_tag(tag: &FrameTag) -> Option<FrameId> {
    books()
        .iter()
        .flat_map(|book| {
            lock(book)
                .tags
                .iter()
                .filter(|(_, found)| found.as_ref() == Some(tag))
                .map(|(frame, _)| *frame)
                .collect::<Vec<_>>()
        })
        .max_by_key(|frame| frame.index())
}
