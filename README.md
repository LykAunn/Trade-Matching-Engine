# Trade Matching Engine

A high-performance order matching engine and trade execution system written in Rust. This engine is designed to
simulate a financial exchange, processing buy and sell orders efficiently with deterministic execution.

## Features

- **High-Performance Execution:** Built in Rust for memory safety and zero-cost abstractions.
- **Custom Order Book:** Utilises optimised structs (`User`, `Order`, `Event`) for tracking order states and
user positions.
- **Efficient Order Management:** Implements $O(1)$ `swap_remove` operations for processing unmatched orders.
- **Event-Driven Architecture:** Trades and market activities are handled via dedicated event streams and price feed channels.

## Architecture

The engine is built around three core components:
- **Order Book:** Maintains the state of all active `Order`s (both matched and unmatched).
- **Users:** Tracks individual user states, including their specific matched/unmatched orders and current position balance.

## Getting Started

### Prerequisites
- [Rust toolchain](https://rustup.rs/)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/LykAunn/Trade-Matching-Engine.git
   cd Trade-Matching-Engine
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the tests:
   ```bash
   cargo test
   ```

## Roadmap
- [x] Basic order submission and matching logic
- [x] Basic Terminal UI
- [ ] User position tracking
- [ ] Implement limit vs market orders
- [ ] Concurrency and multi-threaded order processing