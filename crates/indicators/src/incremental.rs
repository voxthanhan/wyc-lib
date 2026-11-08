//! Low-level append cache over a validated immutable AvailableBar prefix.
//! wyc_lib owns history/identity/revision validation. Each staged delta is atomic.
use crate::{advanced, basic, rolling};
use contracts::{
    AnalysisConfig, AnalysisError, AvailableBar, ContractError, ErrorCode, FiniteF64,
    IndicatorConfig, IndicatorKind as K, MissingReason, NumericSample, NumericValue as V,
};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq)]
struct Smooth {
    count: usize,
    seed: Vec<FiniteF64>,
    previous: Option<f64>,
}
impl Smooth {
    fn feed(
        &mut self,
        current: f64,
        period: usize,
        index: usize,
        offset: usize,
        name: &str,
        wilder: bool,
    ) -> Result<V, AnalysisError> {
        self.count += 1;
        if self.previous.is_none() {
            self.seed.push(FiniteF64::new(current)?);
        }
        if self.count < period {
            return Ok(basic::warmup(period + offset, index));
        }
        let path = format!("indicators.{name}[{index}]");
        let value = if period == 1 {
            current
        } else if let Some(previous) = self.previous {
            basic::smooth_next(previous, current, period, wilder, &path)?
        } else {
            rolling::mean(&self.seed, &path)?
        };
        self.previous = Some(value);
        self.seed.clear();
        basic::ready(value, &path)
    }
}

/// Integration cache; configuration is fixed at creation. No serialized ABI.
#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorMemory {
    config: IndicatorConfig,
    processed: usize,
    basic_only: bool,
    ema: Smooth,
    atr: Smooth,
    rsi_gain: Smooth,
    rsi_loss: Smooth,
    macd_fast: Smooth,
    macd_slow: Smooth,
    macd_signal: Smooth,
    adx_tr: Smooth,
    adx_plus_dm: Smooth,
    adx_minus_dm: Smooth,
    adx: Smooth,
}
impl IndicatorMemory {
    pub fn new(config: &AnalysisConfig) -> Result<Self, AnalysisError> {
        config.validate()?;
        Ok(Self {
            config: config.indicators.clone(),
            processed: 0,
            basic_only: false,
            ema: Smooth::default(),
            atr: Smooth::default(),
            rsi_gain: Smooth::default(),
            rsi_loss: Smooth::default(),
            macd_fast: Smooth::default(),
            macd_slow: Smooth::default(),
            macd_signal: Smooth::default(),
            adx_tr: Smooth::default(),
            adx_plus_dm: Smooth::default(),
            adx_minus_dm: Smooth::default(),
            adx: Smooth::default(),
        })
    }

    pub(crate) fn new_basic(config: &AnalysisConfig) -> Result<Self, AnalysisError> {
        let mut memory = Self::new(config)?;
        memory.basic_only = true;
        Ok(memory)
    }

    /// Prefix must append exactly one validated bar without changing old bars.
    /// Computes only its new samples; self remains unchanged on every result.
    pub fn prepare(
        &self,
        bars: &[AvailableBar],
    ) -> Result<(Self, BTreeMap<K, NumericSample>), AnalysisError> {
        if bars.len() != self.processed + 1 {
            return Err(ContractError::new(ErrorCode::StateMismatch, "indicators.prefix").into());
        }
        let mut next = self.clone();
        let i = self.processed;
        let bar = &bars[i];
        let c = &self.config;
        let p = c.period as usize;
        let fast = c.macd_fast as usize;
        let slow = c.macd_slow as usize;
        let signal = c.macd_signal as usize;
        let mut raw = BTreeMap::new();
        let start = (i + 1).saturating_sub(p);
        let close: Vec<_> = bars[start..].iter().map(|b| b.close).collect();
        let volume: Vec<_> = bars[start..].iter().map(|b| b.volume).collect();
        let window = |name: &str,
                      evaluate: fn(&[FiniteF64], &str) -> Result<f64, AnalysisError>,
                      values: &[FiniteF64]| {
            if i + 1 < p {
                Ok(basic::warmup(p, i))
            } else {
                let path = format!("{name}[{i}]");
                basic::ready(evaluate(values, &path)?, &path)
            }
        };
        let mean = window("rolling.mean", rolling::mean, &close)?;
        let wma = window("indicators.wma", basic::weighted_mean, &close)?;
        let volume_mean = window("rolling.mean", rolling::mean, &volume)?;
        let rvol = match &volume_mean {
            V::Ready { value } if value.get() == 0.0 => V::Missing {
                reason: MissingReason::ZeroDenominator,
            },
            V::Ready { value } => basic::ready(
                bar.volume.get() / value.get(),
                &format!("indicators.rvol[{i}]"),
            )?,
            other => other.clone(),
        };
        let tail = &bars[i.saturating_sub(1)..];
        let high: Vec<_> = tail.iter().map(|b| b.high).collect();
        let low: Vec<_> = tail.iter().map(|b| b.low).collect();
        let closes: Vec<_> = tail.iter().map(|b| b.close).collect();
        let tr = basic::range_at(
            &high,
            &low,
            &closes,
            tail.len() - 1,
            &format!("indicators.tr[{i}]"),
        )?;
        let ema = next.ema.feed(bar.close.get(), p, i, 0, "ema", false)?;
        let atr = next.atr.feed(tr, p, i, 0, "atr", true)?;
        let lag = |percent| {
            if i < p {
                Ok(basic::warmup(p + 1, i))
            } else {
                basic::lag_value(
                    bar.close.get(),
                    bars[i - p].close.get(),
                    percent,
                    &format!(
                        "indicators.{}[{i}]",
                        if percent { "roc" } else { "momentum" }
                    ),
                )
            }
        };
        raw.extend([
            (K::Sma, mean.clone()),
            (K::Ema, ema),
            (K::Wma, wma),
            (K::Tr, basic::ready(tr, &format!("indicators.tr[{i}]"))?),
            (K::Atr, atr),
            (K::Momentum, lag(false)?),
            (K::Roc, lag(true)?),
            (K::VolumeSma, volume_mean),
            (K::Rvol, rvol),
        ]);
        if self.basic_only {
            return finish(next, raw, bar);
        }
        let sum = window("rolling.sum", rolling::sum, &close)?;
        let variance = window("rolling.variance", rolling::variance, &close)?;
        let std = window(
            "rolling.std",
            |v, path| rolling::finite(rolling::variance(v, path)?.sqrt(), path),
            &close,
        )?;
        let min = window(
            "rolling.min",
            |v, _| Ok(rolling::extreme(v, |a, b| a < b)),
            &close,
        )?;
        let max = window(
            "rolling.max",
            |v, _| Ok(rolling::extreme(v, |a, b| a > b)),
            &close,
        )?;
        let macd_fast = next
            .macd_fast
            .feed(bar.close.get(), fast, i, 0, "ema", false)?;
        let macd_slow = next
            .macd_slow
            .feed(bar.close.get(), slow, i, 0, "ema", false)?;
        let line = if i + 1 < slow {
            basic::warmup(slow, i)
        } else {
            basic::ready(
                advanced::difference(
                    number(&macd_fast)?,
                    number(&macd_slow)?,
                    &format!("indicators.macd_line[{i}]"),
                )?,
                &format!("indicators.macd_line[{i}]"),
            )?
        };
        let signal_value = if i + 1 < slow {
            basic::warmup(slow + signal - 1, i)
        } else {
            next.macd_signal
                .feed(number(&line)?, signal, i, slow - 1, "macd_signal", false)?
        };
        let histogram = match &signal_value {
            V::Ready { value } => basic::ready(
                advanced::difference(
                    number(&line)?,
                    value.get(),
                    &format!("indicators.macd_histogram[{i}]"),
                )?,
                &format!("indicators.macd_histogram[{i}]"),
            )?,
            other => other.clone(),
        };
        let mut rsi = basic::warmup(p + 1, i);
        let mut plus = basic::warmup(p + 1, i);
        let mut minus = plus.clone();
        let mut dx = plus.clone();
        let mut adx = basic::warmup(2 * p, i);
        if i > 0 {
            let previous = &bars[i - 1];
            let delta = advanced::difference(
                bar.close.get(),
                previous.close.get(),
                &format!("indicators.rsi[{i}]"),
            )?;
            let gain = next
                .rsi_gain
                .feed(delta.max(0.0), p, i, 1, "rsi_gain", true)?;
            let loss = next
                .rsi_loss
                .feed((-delta).max(0.0), p, i, 1, "rsi_loss", true)?;
            if i >= p {
                rsi = basic::ready(
                    advanced::rsi_ratio(
                        number(&gain)?,
                        number(&loss)?,
                        &format!("indicators.rsi[{i}]"),
                    )?,
                    &format!("indicators.rsi[{i}]"),
                )?;
            }
            let (up, down) = advanced::directional_movement(
                bar.high.get(),
                bar.low.get(),
                previous.high.get(),
                previous.low.get(),
                &format!("indicators.adx_dm[{i}]"),
            )?;
            let range = next.adx_tr.feed(tr, p, i, 1, "adx_tr", true)?;
            let up = next.adx_plus_dm.feed(up, p, i, 1, "adx_plus_dm", true)?;
            let down = next
                .adx_minus_dm
                .feed(down, p, i, 1, "adx_minus_dm", true)?;
            if i >= p {
                let plus_value = advanced::directional_index(
                    number(&up)?,
                    number(&range)?,
                    &format!("indicators.plus_di[{i}]"),
                )?;
                let minus_value = advanced::directional_index(
                    number(&down)?,
                    number(&range)?,
                    &format!("indicators.minus_di[{i}]"),
                )?;
                let dx_value =
                    advanced::dx_ratio(plus_value, minus_value, &format!("indicators.dx[{i}]"))?;
                plus = basic::ready(plus_value, &format!("indicators.plus_di[{i}]"))?;
                minus = basic::ready(minus_value, &format!("indicators.minus_di[{i}]"))?;
                dx = basic::ready(dx_value, &format!("indicators.dx[{i}]"))?;
                adx = next.adx.feed(dx_value, p, i, p, "adx", true)?;
            }
        }
        let (upper, lower) = if i + 1 < p {
            (basic::warmup(p, i), basic::warmup(p, i))
        } else {
            advanced::bands_at(number(&mean)?, number(&std)?, c.bollinger_multiplier, i)?
        };
        raw.extend([
            (K::Sum, sum),
            (K::Min, min),
            (K::Max, max),
            (K::Variance, variance),
            (K::Std, std),
            (K::Rsi, rsi),
            (K::MacdLine, line),
            (K::MacdSignal, signal_value),
            (K::MacdHistogram, histogram),
            (K::PlusDi, plus),
            (K::MinusDi, minus),
            (K::Dx, dx),
            (K::Adx, adx),
            (K::BollingerMiddle, mean),
            (K::BollingerUpper, upper),
            (K::BollingerLower, lower),
        ]);
        finish(next, raw, bar)
    }
}
fn finish(
    mut next: IndicatorMemory,
    raw: BTreeMap<K, V>,
    bar: &AvailableBar,
) -> Result<(IndicatorMemory, BTreeMap<K, NumericSample>), AnalysisError> {
    let mut samples = BTreeMap::new();
    for (kind, result) in raw {
        result.validate()?;
        samples.insert(
            kind,
            NumericSample {
                observed_at: bar.timestamp,
                known_at: bar.known_at,
                result,
            },
        );
    }
    next.processed += 1;
    Ok((next, samples))
}

fn number(value: &V) -> Result<f64, AnalysisError> {
    match value {
        V::Ready { value } => Ok(value.get()),
        _ => Err(ContractError::new(ErrorCode::InvalidResult, "indicators.dependency").into()),
    }
}
