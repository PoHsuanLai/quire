//! The process's thumbnail cache: finished pages by [`ThumbKey`], the oldest dropped first once
//! [`THUMB_CACHE_ENTRIES`] are held. Shared by every `PdfFileThumb` (a launcher's preview and a
//! Quick Look window read the same file at different sizes, each its own entry).

use super::request::{ThumbKey, ThumbRequest};
use ds::PdfPage;
use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex, PoisonError};

/// How many pages the cache holds before it drops the oldest.
pub const THUMB_CACHE_ENTRIES: usize = 64;

#[derive(Debug, Default)]
struct Cache {
    pages: HashMap<ThumbKey, PdfPage>,
    order: VecDeque<ThumbKey>,
}

impl Cache {
    fn insert(&mut self, key: ThumbKey, page: PdfPage) {
        if self.pages.insert(key.clone(), page).is_none() {
            self.order.push_back(key);
        }
        while self.order.len() > THUMB_CACHE_ENTRIES {
            if let Some(oldest) = self.order.pop_front() {
                self.pages.remove(&oldest);
            }
        }
    }
}

static SHARED: LazyLock<Mutex<Cache>> = LazyLock::new(Mutex::default);

/// A cached page for `key`. A poisoned lock (a panic while holding it) still holds whole pages,
/// so its data is used as it is.
pub(crate) fn lookup(key: &ThumbKey) -> Option<PdfPage> {
    let cache = SHARED.lock().unwrap_or_else(PoisonError::into_inner);
    cache.pages.get(key).cloned()
}

pub(crate) fn insert(key: ThumbKey, page: PdfPage) {
    SHARED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(key, page);
}

/// The request's page if it is cached for the file as it is now (reads its modification time,
/// no more), else `None`.
pub fn pdf_thumb_cached(request: &ThumbRequest) -> Option<PdfPage> {
    lookup(&request.key().ok()?)
}

#[cfg(test)]
mod tests {
    use super::{Cache, THUMB_CACHE_ENTRIES};
    use crate::pdf_thumb::{DeviceBox, ThumbKey};
    use ds::{PdfPage, Scale};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn key(n: usize) -> ThumbKey {
        ThumbKey {
            path: PathBuf::from(format!("/tmp/{n}.pdf")),
            modified: SystemTime::UNIX_EPOCH,
            fit: DeviceBox {
                width: 10,
                height: 10,
            },
            scale: Scale::ONE,
        }
    }

    #[test]
    fn the_oldest_page_goes_first_once_full() {
        let mut cache = Cache::default();
        for n in 0..=THUMB_CACHE_ENTRIES {
            cache.insert(key(n), PdfPage::Empty);
        }
        assert_eq!(cache.pages.len(), THUMB_CACHE_ENTRIES);
        assert!(!cache.pages.contains_key(&key(0)));
        assert!(cache.pages.contains_key(&key(THUMB_CACHE_ENTRIES)));
        cache.insert(key(1), PdfPage::Loading);
        assert_eq!(
            cache.order.len(),
            THUMB_CACHE_ENTRIES,
            "a re-insert keeps one slot"
        );
    }
}
