#![no_main]
//! Fuzz the label formulas directly: arbitrary bytes become price arrays and an
//! (index, horizon) pair fed to `forward_return` and `triple_barrier`. Whatever
//! the inputs, the functions must never panic or index out of bounds — an
//! out-of-range request returns `NaN`, never a crash.

use feature_store_core::{forward_return, triple_barrier};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    // First two bytes pick the index and horizon; the rest seed the price path.
    let i = data[0] as usize;
    let h = (data[1] as usize) % 32 + 1; // horizon in 1..=32, never zero
    let closes: Vec<f64> = data[2..]
        .chunks_exact(4)
        .take(256)
        .map(|c| {
            let raw = u32::from_le_bytes(c.try_into().unwrap());
            // Positive, finite, bounded price.
            1.0 + f64::from(raw % 100_000) / 10.0
        })
        .collect();
    if closes.is_empty() {
        return;
    }
    // Derive ordered high/low bands from the closes.
    let highs: Vec<f64> = closes.iter().map(|c| c + 1.0).collect();
    let lows: Vec<f64> = closes.iter().map(|c| (c - 1.0).max(0.01)).collect();

    // Neither call may panic for any (i, h), in-range or not.
    let _ = forward_return(&closes, i, h, false);
    let _ = forward_return(&closes, i, h, true);
    let _ = triple_barrier(&highs, &lows, &closes, i, h, 0.02, 0.02);
});
