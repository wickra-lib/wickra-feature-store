//! The forward-looking label formulas against small, hand-computed cases:
//! forward return (simple and log) and the triple barrier's four outcomes
//! (up-hit, down-hit, the down-wins tie, and the no-touch zero), plus the
//! not-enough-future `NaN` guard.

use feature_store_core::{forward_return, triple_barrier};

const EPS: f64 = 1e-12;

#[test]
fn forward_return_simple() {
    let closes = [100.0, 105.0, 110.0, 121.0];
    // i=0, h=1 -> 105/100 - 1 = 0.05
    assert!((forward_return(&closes, 0, 1, false) - 0.05).abs() < EPS);
    // i=0, h=3 -> 121/100 - 1 = 0.21
    assert!((forward_return(&closes, 0, 3, false) - 0.21).abs() < EPS);
    // i=1, h=2 -> 121/105 - 1
    assert!((forward_return(&closes, 1, 2, false) - (121.0 / 105.0 - 1.0)).abs() < EPS);
}

#[test]
fn forward_return_log() {
    let closes = [100.0, 110.0];
    // ln(110/100)
    assert!((forward_return(&closes, 0, 1, true) - (1.1_f64).ln()).abs() < EPS);
}

#[test]
fn forward_return_missing_future_is_nan() {
    let closes = [100.0, 101.0, 102.0];
    // i=2 has no bar 2+1 -> NaN
    assert!(forward_return(&closes, 2, 1, false).is_nan());
    // horizon reaches exactly the last bar -> defined
    assert!(forward_return(&closes, 0, 2, false).is_finite());
}

#[test]
fn forward_return_zero_base_is_nan() {
    let closes = [0.0, 50.0];
    assert!(forward_return(&closes, 0, 1, false).is_nan());
}

#[test]
fn triple_barrier_up_hit() {
    // entry 100, up 2% -> upper 102; bar 2 high 103 crosses it first (bar 1 does not).
    let highs = [100.0, 101.0, 103.0];
    let lows = [100.0, 99.5, 101.0];
    let closes = [100.0, 100.0, 100.0];
    assert!((triple_barrier(&highs, &lows, &closes, 0, 2, 0.02, 0.02) - 1.0).abs() < EPS);
}

#[test]
fn triple_barrier_down_hit() {
    // entry 100, down 2% -> lower 98; bar 1 low 97 crosses it -> -1.
    let highs = [100.0, 100.5, 101.0];
    let lows = [100.0, 97.0, 99.0];
    let closes = [100.0, 100.0, 100.0];
    assert!((triple_barrier(&highs, &lows, &closes, 0, 2, 0.02, 0.02) + 1.0).abs() < EPS);
}

#[test]
fn triple_barrier_down_wins_the_tie() {
    // bar 1 touches both barriers (high 103, low 97); the lower is checked first
    // so the conservative outcome is -1.
    let highs = [100.0, 103.0];
    let lows = [100.0, 97.0];
    let closes = [100.0, 100.0];
    assert!((triple_barrier(&highs, &lows, &closes, 0, 1, 0.02, 0.02) + 1.0).abs() < EPS);
}

#[test]
fn triple_barrier_no_touch_is_zero() {
    // Neither barrier reached within the horizon -> 0.
    let highs = [100.0, 101.0, 101.5];
    let lows = [100.0, 99.5, 99.0];
    let closes = [100.0, 100.0, 100.0];
    assert!(triple_barrier(&highs, &lows, &closes, 0, 2, 0.02, 0.02).abs() < EPS);
}

#[test]
fn triple_barrier_missing_future_is_nan() {
    let highs = [100.0, 101.0];
    let lows = [100.0, 99.0];
    let closes = [100.0, 100.0];
    // i=1 has no bars in 2..=4 -> NaN.
    assert!(triple_barrier(&highs, &lows, &closes, 1, 3, 0.02, 0.02).is_nan());
}

#[test]
fn triple_barrier_asymmetric_barriers() {
    // up 5% -> 105, down 1% -> 99; bar 1 low 98.5 hits the tight lower barrier
    // first even though the price later rises.
    let highs = [100.0, 102.0, 110.0];
    let lows = [100.0, 98.5, 104.0];
    let closes = [100.0, 100.0, 108.0];
    assert!((triple_barrier(&highs, &lows, &closes, 0, 2, 0.05, 0.01) + 1.0).abs() < EPS);
}
