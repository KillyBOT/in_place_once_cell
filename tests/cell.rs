use in_place_once_cell::InPlaceOnceCell;

#[test]
/// Test basic functionality
fn basic() {
    let c = InPlaceOnceCell::new(34);
    assert!(c.get().is_none());

    assert_eq!(c.get_or_mutate(|v| *v = *v * *v), &1156);
    assert_eq!(c.get(), Some(&1156));
    assert_eq!(c.get_or_mutate(|v| *v = *v + 1), &1156);
}
