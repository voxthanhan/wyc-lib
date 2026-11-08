//! P1.4 indicators with independent component warm-up and causal batch output.
use crate::basic::{ema, range_at, ready, smooth_at, validate_ohlc, validate_period, warmup};
use crate::rolling::{finite, rolling_mean, rolling_std};
use contracts::{
    AnalysisConfig, AnalysisError, AvailableData, ContractError, ErrorCode, FiniteF64,
    IndicatorKind, MarketData, ModuleResult, NumericSample, NumericValue, ResultMetadata,
    CONFIG_VERSION, SCHEMA_VERSION,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct MacdResult {
    pub line: Vec<NumericValue>,
    pub signal: Vec<NumericValue>,
    pub histogram: Vec<NumericValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdxResult {
    pub plus_di: Vec<NumericValue>,
    pub minus_di: Vec<NumericValue>,
    pub dx: Vec<NumericValue>,
    pub adx: Vec<NumericValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BollingerResult {
    pub middle: Vec<NumericValue>,
    pub upper: Vec<NumericValue>,
    pub lower: Vec<NumericValue>,
}

/// Only the eleven P1.4 series, not the complete P1 catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct AdvancedIndicatorResult {
    pub series: BTreeMap<IndicatorKind, Vec<NumericSample>>,
}

pub fn rsi(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError> {
    validate_period(period)?;
    let mut gains = Vec::with_capacity(values.len().saturating_sub(1));
    let mut losses = Vec::with_capacity(gains.capacity());
    for index in 1..values.len() {
        let delta = finite(
            values[index].get() - values[index - 1].get(),
            &format!("indicators.rsi[{index}]"),
        )?;
        gains.push(FiniteF64::new(delta.max(0.0))?);
        losses.push(FiniteF64::new((-delta).max(0.0))?);
    }
    let gains = smooth_at(&gains, period, true, "rsi_gain", 1)?;
    let losses = smooth_at(&losses, period, true, "rsi_loss", 1)?;
    let mut output = prefix_warmup(values.len(), 1, period + 1);
    for (index, (gain, loss)) in gains.into_iter().zip(losses).enumerate() {
        let path = format!("indicators.rsi[{}]", index + 1);
        output.push(match (gain, loss) {
            (NumericValue::Ready { value: gain }, NumericValue::Ready { value: loss }) => {
                ready(rsi_ratio(gain.get(), loss.get(), &path)?, &path)?
            }
            (warmup, _) => warmup,
        });
    }
    Ok(output)
}

pub fn macd(
    values: &[FiniteF64],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Result<MacdResult, AnalysisError> {
    for period in [fast, slow, signal] {
        validate_period(period)?;
    }
    if fast >= slow {
        return Err(ContractError::new(ErrorCode::InvalidConfig, "indicators.macd").into());
    }
    let fast_values = ema(values, fast)?;
    let slow_values = ema(values, slow)?;
    let mut line = Vec::with_capacity(values.len());
    let mut valid_line = Vec::with_capacity(values.len().saturating_sub(slow - 1));
    for (index, (fast, slow_value)) in fast_values.into_iter().zip(slow_values).enumerate() {
        if index + 1 < slow {
            line.push(warmup(slow, index));
        } else {
            let value = difference(
                number(&fast)?,
                number(&slow_value)?,
                &format!("indicators.macd_line[{index}]"),
            )?;
            valid_line.push(FiniteF64::new(value)?);
            line.push(ready(value, &format!("indicators.macd_line[{index}]"))?);
        }
    }
    let required = slow + signal - 1;
    let mut signal_values = prefix_warmup(values.len(), slow - 1, required);
    signal_values.extend(smooth_at(
        &valid_line,
        signal,
        false,
        "macd_signal",
        slow - 1,
    )?);
    let histogram = line
        .iter()
        .zip(&signal_values)
        .enumerate()
        .map(|(index, (line, signal))| match signal {
            NumericValue::Ready { value } => ready(
                difference(
                    number(line)?,
                    value.get(),
                    &format!("indicators.macd_histogram[{index}]"),
                )?,
                &format!("indicators.macd_histogram[{index}]"),
            ),
            other => Ok(other.clone()),
        })
        .collect::<Result<_, AnalysisError>>()?;
    Ok(MacdResult {
        line,
        signal: signal_values,
        histogram,
    })
}

pub fn adx(
    high: &[FiniteF64],
    low: &[FiniteF64],
    close: &[FiniteF64],
    period: usize,
) -> Result<AdxResult, AnalysisError> {
    validate_period(period)?;
    validate_ohlc(high, low, close)?;
    let mut ranges = Vec::with_capacity(close.len().saturating_sub(1));
    let mut plus_dm = Vec::with_capacity(ranges.capacity());
    let mut minus_dm = Vec::with_capacity(ranges.capacity());
    for index in 1..close.len() {
        let path = format!("indicators.adx_dm[{index}]");
        let (plus, minus) = directional_movement(
            high[index].get(),
            low[index].get(),
            high[index - 1].get(),
            low[index - 1].get(),
            &path,
        )?;
        plus_dm.push(FiniteF64::new(plus)?);
        minus_dm.push(FiniteF64::new(minus)?);
        ranges.push(FiniteF64::new(range_at(
            high,
            low,
            close,
            index,
            &format!("indicators.adx_tr[{index}]"),
        )?)?);
    }
    let ranges = smooth_at(&ranges, period, true, "adx_tr", 1)?;
    let plus_dm = smooth_at(&plus_dm, period, true, "adx_plus_dm", 1)?;
    let minus_dm = smooth_at(&minus_dm, period, true, "adx_minus_dm", 1)?;
    let mut plus_di = prefix_warmup(close.len(), 1, period + 1);
    let mut minus_di = plus_di.clone();
    let mut dx = plus_di.clone();
    let mut valid_dx = Vec::with_capacity(close.len().saturating_sub(period));
    for (offset, ((range, plus), minus)) in ranges.iter().zip(plus_dm).zip(minus_dm).enumerate() {
        let index = offset + 1;
        if index < period {
            plus_di.push(warmup(period + 1, index));
            minus_di.push(warmup(period + 1, index));
            dx.push(warmup(period + 1, index));
            continue;
        }
        let tr = number(range)?;
        let plus = directional_index(number(&plus)?, tr, &format!("indicators.plus_di[{index}]"))?;
        let minus = directional_index(
            number(&minus)?,
            tr,
            &format!("indicators.minus_di[{index}]"),
        )?;
        let path = format!("indicators.dx[{index}]");
        let dx_value = dx_ratio(plus, minus, &path)?;
        plus_di.push(ready(plus, &format!("indicators.plus_di[{index}]"))?);
        minus_di.push(ready(minus, &format!("indicators.minus_di[{index}]"))?);
        dx.push(ready(dx_value, &path)?);
        valid_dx.push(FiniteF64::new(dx_value)?);
    }
    let mut adx = prefix_warmup(close.len(), period, 2 * period);
    adx.extend(smooth_at(&valid_dx, period, true, "adx", period)?);
    Ok(AdxResult {
        plus_di,
        minus_di,
        dx,
        adx,
    })
}

pub fn bollinger_bands(
    values: &[FiniteF64],
    period: usize,
    multiplier: FiniteF64,
) -> Result<BollingerResult, AnalysisError> {
    validate_period(period)?;
    if multiplier.get() <= 0.0 {
        return Err(ContractError::new(
            ErrorCode::InvalidConfig,
            "indicators.bollinger_multiplier",
        )
        .into());
    }
    let middle = rolling_mean(values, period)?;
    let std = rolling_std(values, period)?;
    let mut upper = Vec::with_capacity(values.len());
    let mut lower = Vec::with_capacity(values.len());
    for (index, (mean, std)) in middle.iter().zip(std).enumerate() {
        if index + 1 < period {
            upper.push(warmup(period, index));
            lower.push(warmup(period, index));
            continue;
        }
        let (up, down) = bands_at(number(mean)?, number(&std)?, multiplier, index)?;
        upper.push(up);
        lower.push(down);
    }
    Ok(BollingerResult {
        middle,
        upper,
        lower,
    })
}

/// Validate the entire snapshot, then compute exactly the eleven P1.4 series
/// from its available prefix. Metadata and every component's mask are checked.
pub fn calculate_advanced(
    data: &MarketData,
    config: &AnalysisConfig,
    as_of: i64,
) -> Result<ModuleResult<AdvancedIndicatorResult>, AnalysisError> {
    let available = AvailableData::new(data, as_of)?;
    let config_hash = config.config_hash()?;
    let c = &config.indicators;
    let period = c.period as usize;
    let mut high = Vec::with_capacity(available.len());
    let mut low = Vec::with_capacity(available.len());
    let mut close = Vec::with_capacity(available.len());
    for index in 0..available.len() {
        let bar = available.bar(index).ok_or_else(|| {
            ContractError::new(ErrorCode::InvalidResult, "indicators.available_data")
        })?;
        high.push(bar.high);
        low.push(bar.low);
        close.push(bar.close);
    }
    let macd = macd(
        &close,
        c.macd_fast as usize,
        c.macd_slow as usize,
        c.macd_signal as usize,
    )?;
    let adx = adx(&high, &low, &close, period)?;
    let bands = bollinger_bands(&close, period, c.bollinger_multiplier)?;
    let signal_required = (c.macd_slow + c.macd_signal - 1) as usize;
    let raw = [
        (IndicatorKind::Rsi, rsi(&close, period)?, period + 1),
        (IndicatorKind::MacdLine, macd.line, c.macd_slow as usize),
        (IndicatorKind::MacdSignal, macd.signal, signal_required),
        (
            IndicatorKind::MacdHistogram,
            macd.histogram,
            signal_required,
        ),
        (IndicatorKind::PlusDi, adx.plus_di, period + 1),
        (IndicatorKind::MinusDi, adx.minus_di, period + 1),
        (IndicatorKind::Dx, adx.dx, period + 1),
        (IndicatorKind::Adx, adx.adx, 2 * period),
        (IndicatorKind::BollingerMiddle, bands.middle, period),
        (IndicatorKind::BollingerUpper, bands.upper, period),
        (IndicatorKind::BollingerLower, bands.lower, period),
    ];
    let mut series = BTreeMap::new();
    for (kind, values, required) in raw {
        let mut samples = Vec::with_capacity(available.len());
        for (index, value) in values.into_iter().enumerate() {
            if (index + 1 < required && value != warmup(required, index))
                || (index + 1 >= required && !matches!(value, NumericValue::Ready { .. }))
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
        output: AdvancedIndicatorResult { series },
    })
}

pub(crate) fn directional_index(dm: f64, tr: f64, path: &str) -> Result<f64, AnalysisError> {
    if tr == 0.0 {
        Ok(0.0)
    } else {
        finite(finite(100.0 * dm, path)? / tr, path)
    }
}

fn prefix_warmup(len: usize, offset: usize, required: usize) -> Vec<NumericValue> {
    (0..len.min(offset))
        .map(|index| warmup(required, index))
        .collect()
}

fn number(value: &NumericValue) -> Result<f64, AnalysisError> {
    match value {
        NumericValue::Ready { value } => Ok(value.get()),
        _ => Err(ContractError::new(ErrorCode::InvalidResult, "indicators.dependency").into()),
    }
}

pub(crate) fn rsi_ratio(gain: f64, loss: f64, path: &str) -> Result<f64, AnalysisError> {
    let value = if gain == 0.0 && loss == 0.0 {
        50.0
    } else if loss == 0.0 {
        100.0
    } else if gain == 0.0 {
        0.0
    } else {
        let ratio = finite(gain / loss, path)?;
        let denominator = finite(1.0 + ratio, path)?;
        finite(100.0 - finite(100.0 / denominator, path)?, path)?
    };
    Ok(value)
}

pub(crate) fn difference(a: f64, b: f64, path: &str) -> Result<f64, AnalysisError> {
    finite(a - b, path)
}
pub(crate) fn directional_movement(
    high: f64,
    low: f64,
    previous_high: f64,
    previous_low: f64,
    path: &str,
) -> Result<(f64, f64), AnalysisError> {
    let up = finite(high - previous_high, path)?;
    let down = finite(previous_low - low, path)?;
    Ok((
        if up > down && up > 0.0 { up } else { 0.0 },
        if down > up && down > 0.0 { down } else { 0.0 },
    ))
}
pub(crate) fn dx_ratio(plus: f64, minus: f64, path: &str) -> Result<f64, AnalysisError> {
    let denominator = finite(plus + minus, path)?;
    if denominator == 0.0 {
        Ok(0.0)
    } else {
        let difference = finite(plus - minus, path)?.abs();
        finite(finite(100.0 * difference, path)? / denominator, path)
    }
}
pub(crate) fn bands_at(
    mean: f64,
    std: f64,
    multiplier: FiniteF64,
    index: usize,
) -> Result<(NumericValue, NumericValue), AnalysisError> {
    let width = finite(
        multiplier.get() * std,
        &format!("indicators.bollinger_width[{index}]"),
    )?;
    Ok((
        ready(
            mean + width,
            &format!("indicators.bollinger_upper[{index}]"),
        )?,
        ready(
            mean - width,
            &format!("indicators.bollinger_lower[{index}]"),
        )?,
    ))
}
