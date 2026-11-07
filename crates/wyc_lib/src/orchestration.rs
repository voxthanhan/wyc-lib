use crate::{prepare_request, AnalysisError, AnalysisRequest, AnalysisResult, MarketData};
use contracts::{
    AnalysisConfig, AvailableData, ContractError, ErrorCode, IndicatorKind, IndicatorResult,
    Module, ModuleResult, NumericValue, StructureResult,
};
use indicators::{advanced::calculate_advanced, basic::calculate_basic, rolling};

/// Calculate the complete 25-series P1 catalog on the available prefix.
/// The full config keeps hashes identical across single-module and batch calls.
pub fn calculate_indicators(
    data: &MarketData,
    config: &AnalysisConfig,
    as_of: i64,
) -> Result<ModuleResult<IndicatorResult>, AnalysisError> {
    let available = AvailableData::new(data, as_of)?;
    config.validate()?;
    let basic = calculate_basic(data, config, as_of)?;
    let advanced = calculate_advanced(data, config, as_of)?;
    if basic.metadata != advanced.metadata {
        return Err(invalid("indicators.metadata"));
    }
    let mut series = basic.output.series;
    series.extend(advanced.output.series);
    // Extract only the validated prefix; no future OHLCV is used for calculation.
    let close = (0..available.len())
        .map(|i| {
            available
                .bar(i)
                .map(|bar| bar.close)
                .ok_or_else(|| invalid("indicators.close"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let period = config.indicators.period as usize;
    for (kind, values) in [
        (IndicatorKind::Sum, rolling::rolling_sum(&close, period)?),
        (IndicatorKind::Min, rolling::rolling_min(&close, period)?),
        (IndicatorKind::Max, rolling::rolling_max(&close, period)?),
        (
            IndicatorKind::Variance,
            rolling::rolling_variance(&close, period)?,
        ),
        (IndicatorKind::Std, rolling::rolling_std(&close, period)?),
    ] {
        let samples = values
            .into_iter()
            .enumerate()
            .map(|(i, value)| available.numeric_sample(i, value))
            .collect::<Result<Vec<_>, _>>()?;
        series.insert(kind, samples);
    }
    // Exhaustive enum matching below makes new catalog components an explicit
    // contract decision rather than silently omitting them from warm-up checks.
    if series.len() != 25 {
        return Err(invalid("indicators.series"));
    }
    for (kind, samples) in &series {
        available.validate_numeric_series(samples)?;
        let required = match kind {
            IndicatorKind::Tr => 1,
            IndicatorKind::Momentum
            | IndicatorKind::Roc
            | IndicatorKind::Rsi
            | IndicatorKind::PlusDi
            | IndicatorKind::MinusDi
            | IndicatorKind::Dx => period + 1,
            IndicatorKind::Adx => 2 * period,
            IndicatorKind::MacdLine => config.indicators.macd_slow as usize,
            IndicatorKind::MacdSignal | IndicatorKind::MacdHistogram => {
                (config.indicators.macd_slow + config.indicators.macd_signal - 1) as usize
            }
            IndicatorKind::Sum
            | IndicatorKind::Sma
            | IndicatorKind::Ema
            | IndicatorKind::Wma
            | IndicatorKind::Min
            | IndicatorKind::Max
            | IndicatorKind::Variance
            | IndicatorKind::Std
            | IndicatorKind::Atr
            | IndicatorKind::VolumeSma
            | IndicatorKind::Rvol
            | IndicatorKind::BollingerMiddle
            | IndicatorKind::BollingerUpper
            | IndicatorKind::BollingerLower => period,
        };
        for (i, sample) in samples.iter().enumerate() {
            let valid = if i + 1 < required {
                sample.result
                    == NumericValue::Warmup {
                        required,
                        available: i + 1,
                    }
            } else {
                !matches!(sample.result, NumericValue::Warmup { .. })
            };
            if !valid {
                return Err(invalid("indicators.warmup"));
            }
        }
    }
    Ok(ModuleResult {
        metadata: basic.metadata,
        output: IndicatorResult { series },
    })
}

/// Run only the causal structure engine; indicators are not a P1 dependency.
pub fn analyze_structure(
    data: &MarketData,
    config: &AnalysisConfig,
    as_of: i64,
) -> Result<ModuleResult<StructureResult>, AnalysisError> {
    market_structure::calculate_structure(data, config, as_of)
}

/// Execute requested modules in canonical order, aborting atomically on error.
pub fn analyze_batch(request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError> {
    let prepared = prepare_request(request)?;
    let mut result = AnalysisResult {
        metadata: prepared.metadata().clone(),
        indicators: None,
        structure: None,
    };
    for module in prepared.modules() {
        match module {
            Module::Indicators => {
                let module = calculate_indicators(&request.data, &request.config, request.as_of)?;
                if module.metadata != result.metadata {
                    return Err(invalid("indicators.metadata"));
                }
                result.indicators = Some(module.output);
            }
            Module::MarketStructure => {
                let module = analyze_structure(&request.data, &request.config, request.as_of)?;
                if module.metadata != result.metadata {
                    return Err(invalid("structure.metadata"));
                }
                result.structure = Some(module.output);
            }
            _ => return Err(ContractError::new(ErrorCode::UnsupportedModule, "modules").into()),
        }
    }
    Ok(result)
}

fn invalid(path: &str) -> AnalysisError {
    ContractError::new(ErrorCode::InvalidResult, path).into()
}
