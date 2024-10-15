/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/tick-queue
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use tick_id::TickId;
use tick_queue::Queue;

#[derive(Debug, Clone, PartialEq, Eq)] // Debug is needed for asserts in tests
pub enum GameInput {
    #[allow(unused)]
    Jumping(bool),
    #[allow(unused)]
    MoveHorizontal(i32),
}

#[test_log::test]
fn add_step() {
    let mut items = Queue::new(TickId(23));
    items
        .push(TickId(23), GameInput::MoveHorizontal(-2))
        .expect("Expected a move horizontal tick");
    assert_eq!(items.len(), 1);
    assert_eq!(items.front_tick_id().unwrap().value(), 23)
}

#[test_log::test]
fn push_and_pop_step() {
    let mut items = Queue::new(TickId(23));
    items
        .push(TickId(23), GameInput::Jumping(true))
        .expect("Expected a jumping tick");
    items
        .push(TickId(24), GameInput::MoveHorizontal(42))
        .expect("Expected a move horizontal tick");
    assert_eq!(items.len(), 2);
    assert_eq!(items.front_tick_id().unwrap().value(), 23);
    assert_eq!(items.pop().unwrap().item, GameInput::Jumping(true));
    assert_eq!(items.front_tick_id().unwrap().value(), 24);
}

#[test_log::test]
fn default_queue() {
    let mut items = Queue::<GameInput>::default();
    items
        .push(TickId(0), GameInput::Jumping(true))
        .expect("Expected a jumping tick");
    items
        .push(TickId(1), GameInput::MoveHorizontal(42))
        .expect("Expected a move horizontal tick");
    assert_eq!(items.len(), 2);
    items.discard_count(8);
    assert_eq!(items.len(), 0);
}

#[test_log::test]
fn push_and_discard_count() {
    let mut items = Queue::new(TickId(23));
    items
        .push(TickId(23), GameInput::Jumping(true))
        .expect("Expected a jumping tick");
    items
        .push(TickId(24), GameInput::MoveHorizontal(42))
        .expect("Expected a move horizontal tick");
    assert_eq!(items.len(), 2);
    items.discard_count(8);
    assert_eq!(items.len(), 0);
}

#[test_log::test]
fn push_and_discard_up_to_lower() {
    let mut items = Queue::new(TickId(23));
    items
        .push(TickId(23), GameInput::Jumping(true))
        .expect("Expected a jumping tick");
    items
        .push(TickId(24), GameInput::MoveHorizontal(42))
        .expect("Expected a move horizontal tick");
    assert_eq!(items.len(), 2);
    items.discard_up_to(TickId(1));
    assert_eq!(items.len(), 2);
}

#[test_log::test]
fn push_and_discard_up_to_equal() {
    let mut items = Queue::new(TickId(23));
    items
        .push(TickId(23), GameInput::Jumping(true))
        .expect("Expected a jumping tick");
    items
        .push(TickId(24), GameInput::MoveHorizontal(42))
        .expect("Expected a move horizontal tick");
    assert_eq!(items.len(), 2);
    items.discard_up_to(TickId::new(24));
    assert_eq!(items.len(), 1);
}

#[test_log::test]
fn iterator_over_items() {
    let mut items = Queue::new(TickId::new(0));
    items.push(TickId::new(0), "Move 1").unwrap();
    items.push(TickId::new(1), "Move 2").unwrap();
    items.push(TickId::new(2), "Move 3").unwrap();

    let mut iter = items.iter();
    assert_eq!(iter.next().unwrap().item, "Move 1");
    assert_eq!(iter.next().unwrap().item, "Move 2");
    assert_eq!(iter.next().unwrap().item, "Move 3");
    assert!(iter.next().is_none());
}

#[test_log::test]
fn iterator_from_index() {
    let mut items = Queue::default();
    items.push(TickId::new(0), "Move 1").unwrap();
    items.push(TickId::new(1), "Move 2").unwrap();
    items.push(TickId::new(2), "Move 3").unwrap();

    let mut iter = items.iter_index(1); // Start from index 1 (second item)
    assert_eq!(iter.next().unwrap().item, "Move 2");
    assert_eq!(iter.next().unwrap().item, "Move 3");
    assert!(iter.next().is_none());
}

#[test_log::test]
fn iterator_empty_queue() {
    let items: Queue<String> = Queue::default();

    let mut iter = items.iter();
    assert!(iter.next().is_none());
}

#[test_log::test]
fn iterator_from_index_out_of_bounds() {
    let mut items = Queue::default();
    items.push(TickId::new(0), "Move 1").unwrap();
    items.push(TickId::new(1), "Move 2").unwrap();

    let mut iter = items.iter_index(10); // Start index out of bounds
    assert!(iter.next().is_none()); // No items to iterate over
}

#[test_log::test]
fn into_iter() {
    let mut items = Queue::default();
    items.push(TickId::new(0), "Move 1").unwrap();
    items.push(TickId::new(1), "Move 2").unwrap();
    items.push(TickId::new(2), "Move 3").unwrap();

    let mut iter = items.into_iter();
    assert_eq!(iter.next().unwrap().item, "Move 1");
    assert_eq!(iter.next().unwrap().item, "Move 2");
    assert_eq!(iter.next().unwrap().item, "Move 3");
    assert!(iter.next().is_none());
}
