use crate::{order::Event, orderbook::now_ts};
use std::cmp::{min, max};

pub struct Stats {
    pub total_volume: u64,
    pub trade_count: u64,
    pub total_notional: u64,
    pub candles: Vec<Candle>,
    pub series_start_time: u64,
    pub bucket_duration: u64,
    pub current_candle: Option<Candle>
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            total_volume: 0,
            trade_count: 0,
            total_notional: 0,
            candles: Vec::new(),
            series_start_time: now_ts(),
            bucket_duration: 5000000000,
            current_candle: None
        }
    }

    pub fn record_event(&mut self, event: &Event) {
        if let Event::Trade {price, qty, timestamp, ..} = event {
            self.total_volume += qty;
            self.trade_count += 1;
            self.total_notional += price * qty;

            let bucket_index = (timestamp - self.series_start_time) / self.bucket_duration;
            
            match &mut self.current_candle {
                Some(candle) if candle.bucket_index == bucket_index => {
                    // Current bucket, update
                    candle.record_event(*price, *qty);
                }
                Some(_) => {
                    // Different bucket, create new bucket
                    let old = self.current_candle.take().unwrap();
                    self.candles.push(old);
                    let mut new_candle = Candle::new(*price, bucket_index);
                    new_candle.record_event(*price, *qty);
                    self.current_candle = Some(new_candle)
                }
                None => {
                    // First bucket
                    let mut new_candle = Candle::new(*price, bucket_index);
                    new_candle.record_event(*price, *qty);
                    self.current_candle = Some(new_candle);
                }
            }
        }
        
    }

    pub fn vwap(&self) -> Option<f64> {
        if self.total_volume ==0 {
            None
        } else {
            Some(self.total_notional as f64 / self.total_volume as f64)
        }
    }
}

pub struct Candle {
    pub high: u64,
    pub low: u64,
    pub open: u64,
    pub close: u64,
    pub start_time: u64,
    pub volume: u64,
    pub bucket_index: u64
}

impl Candle {
    pub fn new(price:u64, bucket_index: u64) -> Self {
        Candle {
            high: price,
            low: price,
            open: price,
            close: price,
            start_time: now_ts(),
            volume: 0,
            bucket_index
        }
    }

    pub fn record_event(&mut self, price: u64, qty: u64) {
        self.high = max(self.high, price);
        self.low = min(self.low, price);
        self.close = price;
        self.volume += qty;
    }
}