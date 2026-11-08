//! Deterministic, inclusive rolling windows with explicit warm-up records.
//!
//! Each window is recomputed in chronological order. This reference path takes
//! O(N * min(period, N)) time, O(N) output space and O(1) auxiliary space.
//! It neither selects as_of nor skips missing data: inputs are finite values
//! from a prefix already selected by the caller. See docs/algorithms/P1.md.
use contracts::{AnalysisError, ContractError, ErrorCode, FiniteF64, NumericValue};

/// Rolling sum using chronological Neumaier compensated addition.
pub fn rolling_sum(
    values: &[FiniteF64],
    period: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    map_windows(values, period, "sum", sum)
}

/// Rolling arithmetic mean, using the compensated sum divided by period.
pub fn rolling_mean(
    values: &[FiniteF64],
    period: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    map_windows(values, period, "mean", mean)
}

/// Rolling minimum; equal values retain the earliest sample in the window.
pub fn rolling_min(
    values: &[FiniteF64],
    period: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    map_windows(values, period, "min", |window, _| {
        Ok(extreme(window, |value, current| value < current))
    })
}

/// Rolling maximum; equal values retain the earliest sample in the window.
pub fn rolling_max(
    values: &[FiniteF64],
    period: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    map_windows(values, period, "max", |window, _| {
        Ok(extreme(window, |value, current| value > current))
    })
}

/// Two-pass population variance (divisor period, not period - 1).
pub fn rolling_variance(
    values: &[FiniteF64],
    period: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    map_windows(values, period, "variance", variance)
}

/// Population standard deviation, the square root of the two-pass variance.
pub fn rolling_std(
    values: &[FiniteF64],
    period: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    map_windows(values, period, "std", |window, path| {
        finite(variance(window, path)?.sqrt(), path)
    })
}

fn map_windows(
    values: &[FiniteF64],
    period: usize,
    name: &str,
    evaluate: impl Fn(&[FiniteF64], &str) -> Result<f64, AnalysisError>,
) -> Result<Vec<NumericValue>, AnalysisError> {
    if !(1..=1_000_000).contains(&period) {
        return Err(ContractError::new(ErrorCode::InvalidConfig, "rolling.period").into());
    }
    let mut output = Vec::with_capacity(values.len());
    for index in 0..values.len() {
        let available = index + 1;
        let result = if available < period {
            NumericValue::Warmup {
                required: period,
                available,
            }
        } else {
            let path = format!("rolling.{name}[{index}]");
            let value = finite(
                evaluate(&values[available - period..available], &path)?,
                &path,
            )?;
            NumericValue::Ready {
                value: FiniteF64::new(value)?,
            }
        };
        output.push(result);
    }
    Ok(output)
}

pub(crate) fn finite(value: f64, path: &str) -> Result<f64, AnalysisError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(AnalysisError::NumericOverflow { path: path.into() })
    }
}

#[derive(Default)]
pub(crate) struct CompensatedSum {
    total: f64,
    correction: f64,
}

impl CompensatedSum {
    pub(crate) fn add(&mut self, value: f64, path: &str) -> Result<(), AnalysisError> {
        let next = finite(self.total + value, path)?;
        let correction = if self.total.abs() >= value.abs() {
            finite(finite(self.total - next, path)? + value, path)?
        } else {
            finite(finite(value - next, path)? + self.total, path)?
        };
        self.correction = finite(self.correction + correction, path)?;
        self.total = next;
        Ok(())
    }

    pub(crate) fn finish(self, path: &str) -> Result<f64, AnalysisError> {
        finite(self.total + self.correction, path)
    }
}

pub(crate) fn sum(window: &[FiniteF64], path: &str) -> Result<f64, AnalysisError> {
    let mut sum = CompensatedSum::default();
    for value in window {
        sum.add(value.get(), path)?;
    }
    sum.finish(path)
}

pub(crate) fn mean(window: &[FiniteF64], path: &str) -> Result<f64, AnalysisError> {
    finite(sum(window, path)? / window.len() as f64, path)
}

pub(crate) fn variance(window: &[FiniteF64], path: &str) -> Result<f64, AnalysisError> {
    let mean = mean(window, path)?;
    // An exactly constant window has zero variance even when sum / n rounds
    // away from that constant (e.g. three 0.1s). Validate the mean first so
    // intermediate overflow still follows the documented policy.
    if window.iter().all(|value| *value == window[0]) {
        return Ok(0.0);
    }
    let mut squares = CompensatedSum::default();
    for value in window {
        let delta = finite(value.get() - mean, path)?;
        squares.add(finite(delta * delta, path)?, path)?;
    }
    finite(squares.finish(path)? / window.len() as f64, path)
}

pub(crate) fn extreme(window: &[FiniteF64], better: impl Fn(f64, f64) -> bool) -> f64 {
    // map_windows calls evaluation only with a full, nonempty window.
    let mut current = window[0].get();
    for value in &window[1..] {
        if better(value.get(), current) {
            current = value.get();
        }
    }
    current
}
