//! Per-column feature scaling, applied universe-wide over all emitted rows.
//!
//! Scaling touches only feature columns, never labels. Reductions (`mean`,
//! `std`, `min`, `max`) run serially in row order so the floating-point result
//! is identical in every language and in both the parallel and sequential
//! build paths. `NaN` cells stay `NaN` and are excluded from the statistics.

use crate::matrix::FeatureMatrix;
use crate::spec::Scaling;

/// Scale the first `feature_col_count` columns of `matrix` in place.
pub fn apply_scaling(matrix: &mut FeatureMatrix, feature_col_count: usize, scaling: Scaling) {
    let cols = feature_col_count.min(matrix.columns.len());
    for c in 0..cols {
        match scaling {
            Scaling::ZScore => scale_zscore(matrix, c),
            Scaling::MinMax => scale_minmax(matrix, c),
        }
    }
}

fn scale_zscore(matrix: &mut FeatureMatrix, c: usize) {
    let mut sum = 0.0;
    let mut n = 0.0;
    for row in &matrix.data {
        let x = row[c];
        if x.is_finite() {
            sum += x;
            n += 1.0;
        }
    }
    if n == 0.0 {
        return;
    }
    let mean = sum / n;
    let mut sq = 0.0;
    for row in &matrix.data {
        let x = row[c];
        if x.is_finite() {
            let d = x - mean;
            sq += d * d;
        }
    }
    let std_pop = (sq / n).sqrt();
    for row in &mut matrix.data {
        let x = row[c];
        if x.is_finite() {
            row[c] = if std_pop == 0.0 {
                0.0
            } else {
                (x - mean) / std_pop
            };
        }
    }
}

fn scale_minmax(matrix: &mut FeatureMatrix, c: usize) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut any = false;
    for row in &matrix.data {
        let x = row[c];
        if x.is_finite() {
            any = true;
            if x < min {
                min = x;
            }
            if x > max {
                max = x;
            }
        }
    }
    if !any {
        return;
    }
    let span = max - min;
    for row in &mut matrix.data {
        let x = row[c];
        if x.is_finite() {
            row[c] = if span == 0.0 { 0.0 } else { (x - min) / span };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::RowId;

    fn matrix(col_vals: &[f64]) -> FeatureMatrix {
        let mut m = FeatureMatrix::new(vec!["f".into(), "label".into()]);
        for (i, &v) in col_vals.iter().enumerate() {
            m.push_row(
                RowId {
                    symbol: "AAA".into(),
                    ts: i64::try_from(i).unwrap(),
                },
                vec![v, 0.0],
            );
        }
        m
    }

    #[test]
    fn zscore_centers_and_scales() {
        let mut m = matrix(&[1.0, 2.0, 3.0]);
        apply_scaling(&mut m, 1, Scaling::ZScore);
        // mean 2, std_pop = sqrt(2/3); values map to symmetric z-scores.
        assert!((m.data[0][0] + m.data[2][0]).abs() < 1e-12);
        assert!((m.data[1][0]).abs() < 1e-12);
        // label column untouched.
        assert!(m.data[0][1].abs() < 1e-12);
    }

    #[test]
    fn zscore_constant_column_is_zero() {
        let mut m = matrix(&[5.0, 5.0, 5.0]);
        apply_scaling(&mut m, 1, Scaling::ZScore);
        assert!(m.data.iter().all(|r| r[0] == 0.0));
    }

    #[test]
    fn minmax_maps_to_unit_range() {
        let mut m = matrix(&[10.0, 20.0, 30.0]);
        apply_scaling(&mut m, 1, Scaling::MinMax);
        assert!(m.data[0][0].abs() < 1e-12);
        assert!((m.data[2][0] - 1.0).abs() < 1e-12);
        assert!((m.data[1][0] - 0.5).abs() < 1e-12);
    }

    #[test]
    fn minmax_constant_column_is_zero() {
        let mut m = matrix(&[7.0, 7.0]);
        apply_scaling(&mut m, 1, Scaling::MinMax);
        assert!(m.data.iter().all(|r| r[0] == 0.0));
    }

    #[test]
    fn nan_excluded_and_preserved() {
        let mut m = matrix(&[1.0, f64::NAN, 3.0]);
        apply_scaling(&mut m, 1, Scaling::MinMax);
        assert!(m.data[1][0].is_nan());
        assert!(m.data[0][0].abs() < 1e-12);
        assert!((m.data[2][0] - 1.0).abs() < 1e-12);
    }
}
