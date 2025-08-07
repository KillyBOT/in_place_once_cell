use in_place_once_cell::InPlaceOnceLock;
use std::thread;

const U32_INIT: u32 = 34;
const U32_MUTATED: u32 = U32_INIT * U32_INIT;

/// A test mutator
fn u32_square(v: &mut u32) {
    *v = *v * *v;
}

#[test]
/// Test basic functionality of `get` and `get_or_mutate`.
fn basic() {
    let lock = InPlaceOnceLock::new(U32_INIT);
    assert_eq!(lock.get(), None);
    assert_eq!(lock.get_or_mutate(u32_square), &U32_MUTATED);
    assert_eq!(lock.get_or_mutate(|v| *v = *v + 1), &U32_MUTATED);
}

#[test]
/// Test `Sync` (can locks be mutated across threads?)
fn sync() {
    let lock = InPlaceOnceLock::new(U32_INIT);

    thread::scope(|s| {
        s.spawn(|| {
            lock.get_or_mutate(u32_square);
            assert_eq!(lock.get(), Some(&U32_MUTATED));
        });
    });

    assert_eq!(lock.get_or_mutate(|v| *v = *v + 1), &U32_MUTATED);
}

#[test]
/// Test a simple race condition: two threads try to mutate the same lock at once
fn race() {
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

    assert_eq!(l.get_or_mutate(|v| *v = *v + 1), &U32_MUTATED);
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
}
