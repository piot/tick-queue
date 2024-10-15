# 🏃‍♂️ tick-queue

[![Crates.io](https://img.shields.io/crates/v/tick-queue)](https://crates.io/crates/tick-queue)
[![Documentation](https://docs.rs/tick-queue/badge.svg)](https://docs.rs/tick-queue)

`tick-queue` is a Rust library designed to manage a sequence of items in a strick tick order.
Each item is associated with a unique `TickId`, ensuring items are kept in a correct order.

## ✨ Features

- **Step Management**: Queue items with associated `TickId` to ensure correct processing.
- **Iterator Support**: Iterate through items with both standard and indexed iteration.
- **Flexible Item Handling**: Push, pop, and take items from the queue with tick validation.
- **Error Handling**: Robust error handling for cases such as incorrect `TickId` order.

## 🚀 Getting Started

Add `tick-queue` to your `Cargo.toml`:

```toml
[dependencies]
tick-queue = "0.0.1"
```
