use in_place_once_cell::InPlaceOnceCell;
use std::sync::atomic::{AtomicBool, Ordering};

const U32_INIT: u32 = 34;
const U32_MUTATED: u32 = U32_INIT * U32_INIT;

/// A test mutator
const fn u32_square(v: &mut u32) {
    *v = *v * *v;
}
/// Another test mutator
const fn u32_increment(v: &mut u32) {
    *v = *v + 1;
}

#[test]
/// Test basic functionality
fn basic() {
    let c = InPlaceOnceCell::new(34);
    assert!(c.get().is_none());

    assert_eq!(c.get_or_mutate(|v| *v = *v * *v), &1156);
    assert_eq!(c.get(), Some(&1156));
    assert_eq!(c.get_or_mutate(|v| *v = *v + 1), &1156);
}

#[test]
fn drop() {
    static DROPPED: AtomicBool = AtomicBool::new(false);

    struct Droppable;
    impl Drop for Droppable {
        fn drop(&mut self) {
            DROPPED.store(true, Ordering::Release);
        }
    }

    {
        let cell = InPlaceOnceCell::new(Droppable);

        assert!(cell.get().is_none());
        assert!(!DROPPED.load(Ordering::Acquire));

        cell.get_or_mutate(|&mut Droppable| {});
        assert!(cell.get().is_some());
    }

    assert!(DROPPED.load(Ordering::Acquire));
}

#[test]
fn debug_impl() {
    let cell = InPlaceOnceCell::new(U32_INIT);

    assert!(cell.get().is_none());
    assert_eq!(format!("{cell:?}"), "InPlaceOnceCell(<untouched>)");

    assert_eq!(cell.get_or_mutate(u32_square), &U32_MUTATED);
    assert!(cell.get().is_some());

    assert_eq!(
        format!("{:?}", cell),
        format!("InPlaceOnceCell({U32_MUTATED})")
    );
}

#[test]
fn from_impl() {
    let cell = InPlaceOnceCell::from(U32_INIT);
    assert!(cell.get().is_none());
    assert_eq!(cell.get_or_mutate(u32_square), &U32_MUTATED);
}

#[test]
fn eq_impl() {
    let l = InPlaceOnceCell::new(U32_INIT);
    let r = InPlaceOnceCell::new(U32_MUTATED - 1);
    assert!(l == r);
    assert_eq!(l.get_or_mutate(u32_square), r.get_or_mutate(u32_increment));
    assert!(l == r);
}
