//! Label (target) column definitions, their keys, and the deterministic
//! forward-looking formulas that compute them.

use crate::feature::fmt_num;
use serde::{Deserialize, Serialize};

/// One label (target) column.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Label {
    /// Forward return over `horizon` bars: `close[i+h]/close[i] - 1`, or the log
    /// return when `log` is set.
    ForwardReturn {
        /// Forward horizon in bars (must be > 0).
        horizon: usize,
        /// Use the natural-log return instead of the simple return.
        #[serde(default)]
        log: bool,
    },
    /// Triple-barrier label: `+1` if the upper barrier is hit first, `-1` if the
    /// lower barrier is hit first, `0` if neither is hit within `horizon`.
    TripleBarrier {
        /// Forward horizon in bars (must be > 0).
        horizon: usize,
        /// Upper barrier as a fraction of entry (e.g. `0.02` = +2%).
        up: f64,
        /// Lower barrier as a fraction of entry (e.g. `0.02` = -2%).
        down: f64,
    },
}

impl Label {
    /// The canonical column key: `fwd_return(5)` / `fwd_log_return(5)` for
    /// forward returns, `tb(20,0.02,0.02)` for triple barriers.
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Label::ForwardReturn { horizon, log } => {
                if *log {
                    format!("fwd_log_return({horizon})")
                } else {
                    format!("fwd_return({horizon})")
                }
            }
            Label::TripleBarrier { horizon, up, down } => {
                format!("tb({horizon},{},{})", fmt_num(*up), fmt_num(*down))
            }
        }
    }

    /// The forward horizon of this label, in bars.
    #[must_use]
    pub fn horizon(&self) -> usize {
        match self {
            Label::ForwardReturn { horizon, .. } | Label::TripleBarrier { horizon, .. } => *horizon,
        }
    }
}

/// Forward return at bar `i` over horizon `h` for a close series.
///
/// Returns `NaN` when there is not enough future (`i + h > closes.len() - 1`) or
/// when `close[i] == 0`.
#[must_use]
pub fn forward_return(closes: &[f64], i: usize, h: usize, log: bool) -> f64 {
    let m = closes.len();
    if m == 0 || i + h > m - 1 {
        return f64::NAN;
    }
    let base = closes[i];
    if base == 0.0 {
        return f64::NAN;
    }
    let ratio = closes[i + h] / base;
    if log {
        ratio.ln()
    } else {
        ratio - 1.0
    }
}

/// Triple-barrier label at bar `i` over horizon `h`.
///
/// Walks bars `i+1 ..= i+h`; at each bar the lower barrier is checked before the
/// upper one, so a bar touching both resolves to `-1` (the conservative rule).
/// Returns `0` when neither barrier is hit and `NaN` when there is not enough
/// future (`i + h > highs.len() - 1`).
#[must_use]
pub fn triple_barrier(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    i: usize,
    h: usize,
    up: f64,
    down: f64,
) -> f64 {
    let m = closes.len();
    if m == 0 || i + h > m - 1 {
        return f64::NAN;
    }
    let entry = closes[i];
    let upper = entry * (1.0 + up);
    let lower = entry * (1.0 - down);
    for k in (i + 1)..=(i + h) {
        if lows[k] <= lower {
            return -1.0;
        }
        if highs[k] >= upper {
            return 1.0;
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys() {
        assert_eq!(
            Label::ForwardReturn {
                horizon: 5,
                log: false
            }
            .key(),
            "fwd_return(5)"
        );
        assert_eq!(
            Label::ForwardReturn {
                horizon: 5,
                log: true
            }
            .key(),
            "fwd_log_return(5)"
        );
        assert_eq!(
            Label::TripleBarrier {
                horizon: 20,
                up: 0.02,
                down: 0.02
            }
            .key(),
            "tb(20,0.02,0.02)"
        );
    }

    #[test]
    fn forward_return_simple_and_log() {
        let closes = [100.0, 101.0, 102.0, 110.0];
        // i=0, h=3 -> 110/100 - 1
        assert!((forward_return(&closes, 0, 3, false) - 0.10).abs() < 1e-12);
        // log variant
        assert!((forward_return(&closes, 0, 3, true) - (1.1_f64).ln()).abs() < 1e-12);
    }

    #[test]
    fn forward_return_missing_future_is_nan() {
        let closes = [100.0, 101.0];
        assert!(forward_return(&closes, 1, 5, false).is_nan());
    }

    #[test]
    fn forward_return_zero_base_is_nan() {
        let closes = [0.0, 101.0];
        assert!(forward_return(&closes, 0, 1, false).is_nan());
    }

    #[test]
    fn triple_barrier_up_hit() {
        let highs = [100.0, 101.0, 103.0];
        let lows = [100.0, 99.5, 100.0];
        let closes = [100.0, 100.0, 100.0];
        // entry 100, up 0.02 -> upper 102; bar 2 high 103 hits +1
        assert!((triple_barrier(&highs, &lows, &closes, 0, 2, 0.02, 0.02) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn triple_barrier_down_wins_tie() {
        let highs = [100.0, 103.0];
        let lows = [100.0, 97.0];
        let closes = [100.0, 100.0];
        // bar 1 hits both barriers; down wins -> -1
        assert!((triple_barrier(&highs, &lows, &closes, 0, 1, 0.02, 0.02) + 1.0).abs() < 1e-12);
    }

    #[test]
    fn triple_barrier_none_hit_is_zero() {
        let highs = [100.0, 101.0, 101.5];
        let lows = [100.0, 99.5, 99.0];
        let closes = [100.0, 100.0, 100.0];
        assert!(triple_barrier(&highs, &lows, &closes, 0, 2, 0.02, 0.02).abs() < 1e-12);
    }

    #[test]
    fn triple_barrier_missing_future_is_nan() {
        let highs = [100.0, 101.0];
        let lows = [100.0, 99.0];
        let closes = [100.0, 100.0];
        assert!(triple_barrier(&highs, &lows, &closes, 1, 3, 0.02, 0.02).is_nan());
    }
}
