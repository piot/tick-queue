/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/tick-queue
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
/*!
# Tick Queue Crate

The `tick-queue` crate provides utilities for managing a sequence of items.
Each item is associated with a unique tick identifier ([`TickId`]), ensuring that items are processed in the correct order.

The crate offers functionality for pushing items, iterating over them, and managing the internal state of the item queue.
It supports both direct manipulation of the item queue and indexed iteration.

## Example

```rust
use tick_queue::{Queue, ItemInfo};
use tick_id::TickId;

// Create a new Queue instance with an initial tick
let mut queue = Queue::new(TickId::new(0));

// Push items into the queue
queue.push(TickId::new(0), "Step 1").unwrap();
queue.push(TickId::new(1), "Step 2").unwrap();

// Pop the first item
let item = queue.pop();
assert_eq!(item.unwrap().item, "Step 1");

// Iterate over remaining items
for item in queue.iter() {
  println!("Tick {}: {}", item.tick_id, item.item);
}
```

*/

use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use tick_id::TickId;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ItemInfo<T> {
    pub item: T,
    pub tick_id: TickId,
}

impl<T: Display> Display for ItemInfo<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.tick_id, self.item)
    }
}

#[derive(Debug)]
pub struct Queue<T> {
    items: VecDeque<ItemInfo<T>>,
    expected_write_id: TickId, // Tracks the next TickId to be written, ensuring continuity even when the queue is empty
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            expected_write_id: Default::default(),
        }
    }
}

impl<T> Queue<T> {
    pub fn iter(&self) -> impl Iterator<Item = &ItemInfo<T>> {
        self.items.iter()
    }
}

impl<T> IntoIterator for Queue<T> {
    type Item = ItemInfo<T>;
    type IntoIter = std::collections::vec_deque::IntoIter<ItemInfo<T>>;

    /// Consumes the `Queue` collection and returns an iterator over the items.
    ///
    /// This allows the use of the `for` loop and other iterator methods
    /// without manually calling `iter()`.
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub struct FromIndexIterator<'a, T> {
    deque: &'a VecDeque<ItemInfo<T>>,
    #[allow(unused)]
    start_index: usize,
    current_index: usize,
}

impl<'a, T> FromIndexIterator<'a, T> {
    #[must_use]
    pub const fn new(deque: &'a VecDeque<ItemInfo<T>>, start_index: usize) -> Self {
        Self {
            deque,
            start_index,
            current_index: start_index,
        }
    }
}

impl<T: Clone> Iterator for FromIndexIterator<'_, T> {
    type Item = ItemInfo<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.deque.get(self.current_index)?;
        self.current_index += 1;
        Some(item.clone())
    }
}

pub const TICK_ID_MAX: u32 = u32::MAX;

#[derive(Debug)]
pub enum QueueError {
    WrongTickId {
        expected: TickId,
        encountered: TickId,
    },
}

impl<T: Clone> Queue<T> {
    #[must_use]
    pub const fn new(tick_id: TickId) -> Self {
        Self {
            items: VecDeque::new(),
            expected_write_id: tick_id,
        }
    }

    /// Clears the queue and resets the expected read and write tick IDs.
    pub fn clear(&mut self, initial_tick_id: TickId) {
        self.items.clear();
        self.expected_write_id = initial_tick_id;
    }

    /// Pushes an item into the queue at the specified `TickId`.
    ///
    /// This method ensures that the item is added at the correct position in the tick sequence. The
    /// `tick_id` must match the expected `TickId` for the queue to maintain an unbroken sequence.
    ///
    /// # Parameters
    /// - `tick_id`: The `TickId` where the item should be inserted. It must match the queue's expected next `TickId`.
    /// - `item`: The item to be inserted into the queue.
    ///
    /// # Returns
    /// - `Ok(())` if the item is successfully added to the queue.
    /// - `Err(QueueError)` if the provided `tick_id` does not match the expected `TickId`.
    ///
    /// # Errors
    /// - Returns a `QueueError::WrongTickId` if the `tick_id` provided does not match the expected
    ///   `TickId`, which maintains the sequential order of the queue.
    ///
    pub fn push(&mut self, tick_id: TickId, item: T) -> Result<(), QueueError> {
        if self.expected_write_id != tick_id {
            Err(QueueError::WrongTickId {
                expected: self.expected_write_id,
                encountered: tick_id,
            })?;
        }

        self.push_internal(item);

        Ok(())
    }

    fn push_internal(&mut self, item: T) {
        let info = ItemInfo {
            item,
            tick_id: self.expected_write_id,
        };
        self.items.push_back(info);
        self.expected_write_id += 1;
    }

    #[must_use]
    pub fn debug_get(&self, index: usize) -> Option<&ItemInfo<T>> {
        self.items.get(index)
    }

    #[must_use]
    pub fn pop(&mut self) -> Option<ItemInfo<T>> {
        self.items.pop_front()
    }

    pub fn discard_up_to(&mut self, tick_id: TickId) {
        while let Some(info) = self.items.front() {
            if info.tick_id >= tick_id {
                break;
            }

            self.items.pop_front();
        }
    }

    pub fn discard_count(&mut self, count: usize) {
        if count >= self.items.len() {
            self.items.clear();
        } else {
            self.items.drain(..count);
        }
    }

    /// Pops up to a certain amount of items from the front of the queue and returns
    /// the first `TickId` and a vector of `T`. Returns `None` if the queue
    /// is empty.
    ///
    /// # Parameters
    /// - `count`: The number of items to pop (or fewer if not enough items are available).
    ///
    /// # Returns
    /// - `Some((TickId, Vec<T>))` if there are items available.
    /// - `None` if the queue is empty.
    ///
    /// # Example
    /// ```rust
    /// use tick_id::TickId;
    /// use tick_queue::Queue;
    /// let mut items = Queue::new(TickId::new(0));
    /// items.push(TickId::new(0), "Step 1").unwrap();
    /// items.push(TickId::new(1), "Step 2").unwrap();
    ///
    /// let result = items.take(5);  // Will return up to 5 items (in this case 2)
    /// if let Some((tick_id, popped_items)) = result {
    ///     assert_eq!(tick_id, TickId::new(0));
    ///     assert_eq!(popped_items, vec!["Step 1", "Step 2"]);
    /// }
    /// ```
    #[must_use]
    pub fn take(&mut self, count: usize) -> Option<(TickId, Vec<T>)> {
        let first_tick_id = self.front_tick_id()?;

        let items_to_take: Vec<T> = self
            .items
            .drain(..count.min(self.items.len()))
            .map(|item_info| item_info.item)
            .collect();

        Some((first_tick_id, items_to_take))
    }

    #[must_use]
    pub fn front_tick_id(&self) -> Option<TickId> {
        self.items.front().map(|item_info| item_info.tick_id)
    }

    #[must_use]
    pub const fn expected_write_tick_id(&self) -> TickId {
        self.expected_write_id
    }

    #[must_use]
    pub fn back_tick_id(&self) -> Option<TickId> {
        self.items.back().map(|item_info| item_info.tick_id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<T> {
        let (front_slice, back_slice) = self.items.as_slices();
        front_slice
            .iter()
            .chain(back_slice.iter())
            .map(|item_info| item_info.item.clone())
            .collect()
    }

    #[must_use]
    pub const fn iter_index(&self, start_index: usize) -> FromIndexIterator<T> {
        FromIndexIterator::new(&self.items, start_index)
    }
}
