//! A newer command supersedes the one being thought about (Julien 2026-11-27: the bot froze
//! behind one long solve while his clicks queued). The caller names what "current" means;
//! learning asks between expansions and stops, so thinking never outlives its command.
use std::cell::RefCell;

thread_local! {static CHECK: RefCell<Option<Box<dyn Fn() -> bool>>> = const { RefCell::new(None) };}

/// Runs `f` while `stale()` says whether its command has been superseded.
pub fn while_current<T>(stale: impl Fn() -> bool + 'static, f: impl FnOnce() -> T) -> T {
    CHECK.set(Some(Box::new(stale)));
    let out = f();
    CHECK.set(None);
    out
}

/// True when the command being thought about is no longer the latest.
pub(crate) fn superseded() -> bool {
    CHECK.with_borrow(|c| c.as_ref().is_some_and(|stale| stale()))
}
