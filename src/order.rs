use std::time::{Duration, UNIX_EPOCH};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeType {
    Limit,
    Market,
    Ioc,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub trade_type: TradeType,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Accepted {
        order_id: u64,
        timestamp: u64,
    },

    Trade {
        buy_order_id: u64,
        sell_order_id: u64,
        price: u64,
        qty: u64,
        timestamp: u64,
    },

    Rested {
        order_id: u64,
        timestamp: u64,
    },

    Cancelled {
        order_id: u64,
        timestamp: u64,
    },

    Rejected {
        order_id: u64,
        reason: String,
        timestamp: u64,
    },

    RestingFulfilled {
        order_id: u64,
        timestamp: u64,
    },

    Discarded {
        order_id: u64,
        timestamp: u64,
        quantity: u64,
    }
    
}

fn format_timestamp(nanos: u64) -> String {
    let duration = Duration::from_nanos(nanos);
    let datetime: DateTime<Utc> = (UNIX_EPOCH + duration).into();
    datetime.format("%Y-%m-%d %H-%M:%S%.3f").to_string()
}

impl Event {

    pub fn to_log_line(&self) -> String {
        match self {
            Event::Accepted { order_id, timestamp } => {
                format!("[{}] ACCEPT    order #{}", format_timestamp(*timestamp), order_id)
            }
            
            Event::Trade { buy_order_id, sell_order_id, price, qty, timestamp } => {
                format!("[{}] TRADE      {} @ {} (Buy #{} / Sell #{})",
                format_timestamp(*timestamp), qty, price, buy_order_id, sell_order_id
                )
            }

            Event::Rested { order_id, timestamp } => {
                format!("[{}] REST    order #{}",
                format_timestamp(*timestamp), order_id
                )
            }

            Event::Cancelled { order_id, timestamp } => {
                format!("[{}] CANCELLED     order #{}",
                format_timestamp(*timestamp), order_id
                )
            }

            Event::Rejected { order_id, reason, timestamp } => {
                format!("[{}] REJECTED     order #{} ({})",
                format_timestamp(*timestamp), order_id, reason
            )
            }

            Event::RestingFulfilled { order_id, timestamp } => {
                format!("[{}] REST FULFILLED     order #{}",
                format_timestamp(*timestamp), order_id
            )
            }

            Event::Discarded { order_id, timestamp, quantity } => {
                format!("[{}] REMAINING DISCARDED     order #{} with {} units remaining",
                format_timestamp(*timestamp), order_id, quantity)
            }
        }
    }
}