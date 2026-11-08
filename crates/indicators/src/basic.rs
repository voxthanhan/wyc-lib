//! P1.3 reference indicators. Slice APIs require an already causal prefix.
//! `calculate_basic` selects and validates market-level availability itself.
use crate::rolling::{finite, mean, rolling_mean, CompensatedSum};
use contracts::{
    AnalysisConfig, AnalysisError, AvailableData, ContractError, ErrorCode, FiniteF64,
    IndicatorKind, MarketData, MissingReason, ModuleResult, NumericSample, NumericValue,
    ResultMetadata, CONFIG_VERSION, SCHEMA_VERSION,
};
use std::collections::BTreeMap;

/// The nine P1.3 series; this is not the complete P1 indicator catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicIndicatorResult {
    pub series: BTreeMap<IndicatorKind, Vec<NumericSample>>,
}

pub fn sma(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError> {
    validate_period(period)?;
    rolling_mean(values, period)
}

/// SMA-seeded EMA; period one is the identity, including at extreme scales.
pub fn ema(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError> {
    smooth(values, period, false, "ema")
}

pub fn wma(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError> {
    validate_period(period)?;
    let mut output = Vec::with_capacity(values.len());
    for index in 0..values.len() {
        if index + 1 < period {
            output.push(warmup(period, index));
            continue;
        }
        let path = format!("indicators.wma[{index}]");
        output.push(ready(
            weighted_mean(&values[index + 1 - period..=index], &path)?,
            &path,
        )?);
    }
    Ok(output)
}

pub fn true_range(
    high: &[FiniteF64],
    low: &[FiniteF64],
    close: &[FiniteF64],
) -> Result<Vec<NumericValue>, AnalysisError> {
    validate_ohlc(high, low, close)?;
    let mut output = Vec::with_capacity(close.len());
    for index in 0..close.len() {
        let path = format!("indicators.tr[{index}]");
        output.push(ready(range_at(high, low, close, index, &path)?, &path)?);
    }
    Ok(output)
}

pub(crate) fn range_at(
    high: &[FiniteF64],
    low: &[FiniteF64],
    close: &[FiniteF64],
    index: usize,
    path: &str,
) -> Result<f64, AnalysisError> {
    let range = finite(high[index].get() - low[index].get(), path)?;
    if index == 0 {
        return Ok(range);
    }
    let high_gap = finite(high[index].get() - close[index - 1].get(), path)?.abs();
    let low_gap = finite(low[index].get() - close[index - 1].get(), path)?.abs();
    Ok(range.max(high_gap).max(low_gap))
}

pub fn atr(
    high: &[FiniteF64],
    low: &[FiniteF64],
    close: &[FiniteF64],
    period: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    validate_period(period)?;
    let ranges = true_range(high, low, close)?;
    atr_from_ranges(&ranges, period)
}

pub fn momentum(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError> {
    lagged(values, period, false)
}

pub fn roc(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError> {
    lagged(values, period, true)
}

pub fn volume_sma(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError> {
    validate_period(period)?;
    validate_volume(values)?;
    sma(values, period)
}

pub fn rvol(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError> {
    let averages = volume_sma(values, period)?;
    relative_volume(values, averages)
}

/// Validate the snapshot and config, calculate only the available prefix, and
/// attach lineage to every warm-up/missing/ready record. No caller mutation.
pub fn calculate_basic(
    data: &MarketData,
    config: &AnalysisConfig,
    as_of: i64,
) -> Result<ModuleResult<BasicIndicatorResult>, AnalysisError> {
    let available = AvailableData::new(data, as_of)?;
    let config_hash = config.config_hash()?;
    let period = config.indicators.period as usize;
    let mut high = Vec::with_capacity(available.len());
    let mut low = Vec::with_capacity(available.len());
    let mut close = Vec::with_capacity(available.len());
    let mut volume = Vec::with_capacity(available.len());
    for index in 0..available.len() {
        let bar = available.bar(index).ok_or_else(|| {
            ContractError::new(ErrorCode::InvalidResult, "indicators.available_data")
        })?;
        high.push(bar.high);
        low.push(bar.low);
        close.push(bar.close);
        volume.push(bar.volume);
    }
    let ranges = true_range(&high, &low, &close)?;
    let atr_values = atr_from_ranges(&ranges, period)?;
    let volume_averages = volume_sma(&volume, period)?;
    let relative_volumes = relative_volume(&volume, volume_averages.clone())?;
    let raw = [
        (IndicatorKind::Sma, sma(&close, period)?, period),
        (IndicatorKind::Ema, ema(&close, period)?, period),
        (IndicatorKind::Wma, wma(&close, period)?, period),
        (IndicatorKind::Tr, ranges, 1),
        (IndicatorKind::Atr, atr_values, period),
        (
            IndicatorKind::Momentum,
            momentum(&close, period)?,
            period + 1,
        ),
        (IndicatorKind::Roc, roc(&close, period)?, period + 1),
        (IndicatorKind::VolumeSma, volume_averages, period),
        (IndicatorKind::Rvol, relative_volumes, period),
    ];
    let mut series = BTreeMap::new();
    for (kind, values, required) in raw {
        let mut samples = Vec::with_capacity(available.len());
        for (index, value) in values.into_iter().enumerate() {
            if (index + 1 < required && value != warmup(required, index))
                || (index + 1 >= required && matches!(value, NumericValue::Warmup { .. }))
            {
                return Err(
                    ContractError::new(ErrorCode::InvalidResult, "indicators.warmup").into(),
                );
            }
            samples.push(available.numeric_sample(index, value)?);
        }
        available.validate_numeric_series(&samples)?;
        series.insert(kind, samples);
    }
    Ok(ModuleResult {
        metadata: ResultMetadata {
            schema_version: SCHEMA_VERSION.into(),
            engine_version: env!("CARGO_PKG_VERSION").into(),
            algorithm_version: CONFIG_VERSION.into(),
            symbol: data.symbol.clone(),
            timeframe: data.timeframe.clone(),
            data_revision: data.metadata.data_revision.clone(),
            config_hash,
            as_of,
        },
        output: BasicIndicatorResult { series },
    })
}

pub(crate) fn validate_period(period: usize) -> Result<(), AnalysisError> {
    if !(1..=1_000_000).contains(&period) {
        return Err(ContractError::new(ErrorCode::InvalidConfig, "indicators.period").into());
    }
    Ok(())
}

pub(crate) fn warmup(required: usize, index: usize) -> NumericValue {
    NumericValue::Warmup {
        required,
        available: index + 1,
    }
}

pub(crate) fn ready(value: f64, path: &str) -> Result<NumericValue, AnalysisError> {
    Ok(NumericValue::Ready {
        value: FiniteF64::new(finite(value, path)?)?,
    })
}

fn zero_denominator() -> NumericValue {
    NumericValue::Missing {
        reason: MissingReason::ZeroDenominator,
    }
}

fn smooth(
    values: &[FiniteF64],
    period: usize,
    wilder: bool,
    name: &str,
) -> Result<Vec<NumericValue>, AnalysisError> {
    smooth_at(values, period, wilder, name, 0)
}

/// Smooth a compact dependency series, reporting warm-up and errors relative
/// to its position in the original input. Prefix records are added by caller.
pub(crate) fn smooth_at(
    values: &[FiniteF64],
    period: usize,
    wilder: bool,
    name: &str,
    offset: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    validate_period(period)?;
    let mut output = Vec::with_capacity(values.len());
    let mut previous = None;
    for (index, current) in values.iter().enumerate() {
        if index + 1 < period {
            output.push(warmup(period + offset, index + offset));
            continue;
        }
        let path = format!("indicators.{name}[{}]", index + offset);
        let value = if period == 1 {
            current.get()
        } else if let Some(previous) = previous {
            smooth_next(previous, current.get(), period, wilder, &path)?
        } else {
            mean(&values[..period], &path)?
        };
        output.push(ready(value, &path)?);
        previous = Some(value);
    }
    Ok(output)
}

fn atr_from_ranges(
    ranges: &[NumericValue],
    period: usize,
) -> Result<Vec<NumericValue>, AnalysisError> {
    let values = ranges
        .iter()
        .map(|value| match value {
            NumericValue::Ready { value } => Ok(*value),
            _ => Err(ContractError::new(ErrorCode::InvalidResult, "indicators.tr").into()),
        })
        .collect::<Result<Vec<_>, AnalysisError>>()?;
    smooth(&values, period, true, "atr")
}

fn lagged(
    values: &[FiniteF64],
    period: usize,
    percent: bool,
) -> Result<Vec<NumericValue>, AnalysisError> {
    validate_period(period)?;
    let name = if percent { "roc" } else { "momentum" };
    let mut output = Vec::with_capacity(values.len());
    for index in 0..values.len() {
        if index < period {
            output.push(warmup(period + 1, index));
            continue;
        }
        let path = format!("indicators.{name}[{index}]");
        let previous = values[index - period].get();
        output.push(lag_value(values[index].get(), previous, percent, &path)?);
    }
    Ok(output)
}

fn relative_volume(
    values: &[FiniteF64],
    averages: Vec<NumericValue>,
) -> Result<Vec<NumericValue>, AnalysisError> {
    averages
        .into_iter()
        .enumerate()
        .map(|(index, average)| match average {
            NumericValue::Ready { value } => {
                if value.get() == 0.0 {
                    Ok(zero_denominator())
                } else {
                    ready(
                        values[index].get() / value.get(),
                        &format!("indicators.rvol[{index}]"),
                    )
                }
            }
            other => Ok(other),
        })
        .collect()
}

fn validate_volume(values: &[FiniteF64]) -> Result<(), AnalysisError> {
    for (index, value) in values.iter().enumerate() {
        if value.get() < 0.0 {
            return Err(ContractError::new(
                ErrorCode::NegativeVolume,
                format!("indicators.volume[{index}]"),
            )
            .into());
        }
    }
    Ok(())
}

pub(crate) fn validate_ohlc(
    high: &[FiniteF64],
    low: &[FiniteF64],
    close: &[FiniteF64],
) -> Result<(), AnalysisError> {
    for (name, len) in [("high", high.len()), ("low", low.len())] {
        if len != close.len() {
            return Err(ContractError::new(
                ErrorCode::LengthMismatch,
                format!("indicators.{name}"),
            )
            .into());
        }
    }
    for index in 0..close.len() {
        if low[index] > high[index] || close[index] < low[index] || close[index] > high[index] {
            return Err(ContractError::new(
                ErrorCode::InvalidOhlc,
                format!("indicators.ohlc[{index}]"),
            )
            .into());
        }
    }
    Ok(())
}

pub(crate) fn smooth_next(
    previous: f64,
    current: f64,
    period: usize,
    wilder: bool,
    path: &str,
) -> Result<f64, AnalysisError> {
    let delta = finite(current - previous, path)?;
    let change = finite(
        if wilder {
            delta / period as f64
        } else {
            (2.0 / (period as f64 + 1.0)) * delta
        },
        path,
    )?;
    finite(previous + change, path)
}
pub(crate) fn weighted_mean(window: &[FiniteF64], path: &str) -> Result<f64, AnalysisError> {
    let n = window.len() as f64;
    let mut sum = CompensatedSum::default();
    for (weight, value) in window.iter().enumerate() {
        sum.add(finite((weight + 1) as f64 * value.get(), path)?, path)?;
    }
    Ok(sum.finish(path)? / (n * (n + 1.0) / 2.0))
}
pub(crate) fn lag_value(
    current: f64,
    previous: f64,
    percent: bool,
    path: &str,
) -> Result<NumericValue, AnalysisError> {
    if percent && previous == 0.0 {
        return Ok(zero_denominator());
    }
    let value = if percent {
        let ratio = finite(current / previous, path)?;
        finite(finite(ratio - 1.0, path)? * 100.0, path)?
    } else {
        finite(current - previous, path)?
    };
    ready(value, path)
}
