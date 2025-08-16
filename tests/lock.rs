use in_place_once_cell::InPlaceOnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

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
/// Test basic functionality of `get` and `get_or_mutate`.
fn basic_single() {
    let lock = InPlaceOnceLock::new(U32_INIT);
    assert_eq!(lock.get(), None);
    assert_eq!(lock.get_or_mutate(u32_square), &U32_MUTATED);
    assert_eq!(lock.get_or_mutate(u32_increment), &U32_MUTATED);
}

#[test]
/// Test functionality of `get` and `get_or_mutate` across threads;
fn basic_multi() {
    let lock = InPlaceOnceLock::new(U32_INIT);
    assert_eq!(lock.get(), None);

    thread::scope(|s| {
        s.spawn(|| {
            assert_eq!(lock.get_or_mutate(u32_square), &U32_MUTATED);
            assert_eq!(lock.get_or_mutate(u32_increment), &U32_MUTATED);
            assert_eq!(lock.get(), Some(&U32_MUTATED));
        });
    });

    assert_eq!(lock.get_or_mutate(u32_increment), &U32_MUTATED);
    assert_eq!(lock.get(), Some(&U32_MUTATED));
}

#[test]
/// Test a simple race condition: two threads try to mutate the same lock at once
fn get_or_mutate_race() {
    let l = InPlaceOnceLock::new(U32_INIT);
    assert_eq!(l.get(), None);

    thread::scope(|s| {
        s.spawn(|| {
            l.get_or_mutate(u32_square);
            assert_eq!(l.get(), Some(&U32_MUTATED));
        });
        s.spawn(|| {
            l.get_or_mutate(u32_square);
            assert_eq!(l.get(), Some(&U32_MUTATED));
        });
    });
}

#[test]
/// A bunch of threads mutate a bunch of locks
fn stress() {
    use std::iter;

    const NUM_THREADS: usize = 1024;
    const NUM_ONCES: usize = 1024;

    let locks: Vec<_> = iter::repeat_with(|| InPlaceOnceLock::new(U32_INIT))
        .take(NUM_ONCES)
        .collect();

    for lock in &locks {
        assert!(lock.get().is_none());
    }

    thread::scope(|s| {
        for _ in 0..NUM_THREADS {
            s.spawn(|| {
                for lock in &locks {
                    assert_eq!(lock.get_or_mutate(u32_square), &U32_MUTATED)
                }
            });
        }
    });

    for lock in &locks {
        assert_eq!(lock.get(), Some(&U32_MUTATED));
    }
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

    let lock = InPlaceOnceLock::new(Droppable);
    thread::scope(|s| {
        s.spawn(move || {
            assert!(lock.get().is_none());
            assert!(!DROPPED.load(Ordering::Acquire));

            lock.get_or_mutate(|&mut Droppable| {});
            assert!(lock.get().is_some());

            // `lock` gets dropped here due to the `move`
        });
    });

    assert!(DROPPED.load(Ordering::Acquire));
}

#[test]
fn debug_impl() {
    let lock = InPlaceOnceLock::new(U32_INIT);

    assert!(lock.get().is_none());
    assert_eq!(format!("{lock:?}"), "InPlaceOnceLock(<untouched>)");

    assert_eq!(lock.get_or_mutate(u32_square), &U32_MUTATED);
    assert!(lock.get().is_some());

    assert_eq!(
        format!("{:?}", lock),
        format!("InPlaceOnceLock({U32_MUTATED})")
    );
}

#[test]
fn from_impl() {
    let lock = InPlaceOnceLock::from(U32_INIT);
    assert!(lock.get().is_none());
    assert_eq!(lock.get_or_mutate(u32_square), &U32_MUTATED);
}

#[test]
fn eq_impl() {
    let l = InPlaceOnceLock::new(U32_INIT);
    let r = InPlaceOnceLock::new(U32_MUTATED - 1);
    assert!(l == r);
    assert_eq!(l.get_or_mutate(u32_square), r.get_or_mutate(u32_increment));
    assert!(l == r);
}

#[test]
/// Test that `InPlaceOnceLock` is `Sync` and `Send`.
fn assert_sync_and_send() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<InPlaceOnceLock<Vec<u8>>>();
    assert_sync::<InPlaceOnceLock<Vec<u8>>>();
}
