use contracts::{
    AnalysisError, AvailableData, ContractError, ErrorCode, FiniteF64, MarketData, NumericValue,
};
use indicators::rolling::*;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, error::Error};

type TestResult = Result<(), Box<dyn Error>>;
type RollingFn = fn(&[FiniteF64], usize) -> Result<Vec<NumericValue>, AnalysisError>;

const OPERATIONS: [(&str, RollingFn); 6] = [
    ("sum", rolling_sum),
    ("mean", rolling_mean),
    ("min", rolling_min),
    ("max", rolling_max),
    ("variance", rolling_variance),
    ("std", rolling_std),
];

fn numbers(values: &[f64]) -> Result<Vec<FiniteF64>, ContractError> {
    values.iter().copied().map(FiniteF64::new).collect()
}

fn ready(value: &NumericValue) -> f64 {
    match value {
        NumericValue::Ready { value } => value.get(),
        other => panic!("expected ready, got {other:?}"),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    fixture_id: String,
    algorithm_version: String,
    config: Value,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    period: usize,
    values: Vec<FiniteF64>,
}

#[test]
fn independently_authored_goldens_cover_all_six_operations() -> TestResult {
    let fixture: Fixture = serde_json::from_str(include_str!("fixtures/p1-rolling.json"))?;
    let golden: Value = serde_json::from_str(include_str!("golden/p1-rolling.json"))?;
    assert_eq!(fixture.fixture_id, golden["fixture_id"]);
    assert_eq!(fixture.algorithm_version, golden["algorithm_version"]);
    assert_eq!(fixture.config["periods"], serde_json::json!([1, 2, 3]));
    let cases = golden["cases"].as_array().ok_or("missing golden cases")?;
    assert_eq!(fixture.cases.len(), cases.len());
    for (case, expected) in fixture.cases.iter().zip(cases) {
        assert_eq!(case.id, expected["id"]);
        let before = case.values.clone();
        for (name, operation) in OPERATIONS {
            let actual = operation(&case.values, case.period)?;
            let values: Vec<Option<f64>> = serde_json::from_value(expected[name].clone())?;
            assert_eq!(actual.len(), case.values.len());
            assert_eq!(actual.len(), values.len());
            for (index, (sample, expected)) in actual.iter().zip(values).enumerate() {
                match expected {
                    None => assert_eq!(
                        *sample,
                        NumericValue::Warmup {
                            required: case.period,
                            available: index + 1,
                        }
                    ),
                    Some(expected) => {
                        let actual = ready(sample);
                        if name == "min" || name == "max" {
                            assert_eq!(actual, expected);
                        } else {
                            let rtol = if name == "variance" { 1e-11 } else { 1e-12 };
                            // Tiny values get a stricter relative-only check so atol cannot hide zero.
                            let atol = if case.id == "tiny" { 0.0 } else { 1e-12 };
                            assert!(
                                (actual - expected).abs()
                                    <= atol + rtol * actual.abs().max(expected.abs()),
                                "{}.{name}[{index}]: {actual} != {expected}",
                                case.id
                            );
                        }
                    }
                }
                sample.validate()?;
            }
        }
        assert_eq!(case.values, before);
    }
    Ok(())
}

#[test]
fn empty_short_and_first_valid_boundaries_include_maximum_period() -> TestResult {
    for period in [1, 2, 3, 14, 1_000_000] {
        for (_, operation) in OPERATIONS {
            assert!(operation(&[], period)?.is_empty());
            let len = if period == 1_000_000 { 3 } else { period + 1 };
            let input = numbers(&vec![2.0; len])?;
            let output = operation(&input, period)?;
            for (index, sample) in output.iter().enumerate() {
                if index + 1 < period {
                    assert_eq!(
                        *sample,
                        NumericValue::Warmup {
                            required: period,
                            available: index + 1
                        }
                    );
                } else {
                    assert!(matches!(sample, NumericValue::Ready { .. }));
                }
            }
            assert_eq!(output.len(), len);
        }
    }
    Ok(())
}

#[test]
fn invalid_periods_fail_even_on_empty_input_and_errors_preserve_codes() -> TestResult {
    for period in [0, 1_000_001, usize::MAX] {
        for (_, operation) in OPERATIONS {
            for input in [vec![], numbers(&[1.0])?] {
                let error = operation(&input, period)
                    .err()
                    .ok_or("expected invalid period")?;
                assert_eq!(error.code(), ErrorCode::InvalidConfig);
                assert_eq!(error.path(), "rolling.period");
                assert!(error.source().is_some());
            }
        }
    }
    let error: AnalysisError = ContractError::new(ErrorCode::MissingData, "close[0]").into();
    assert_eq!(error.code(), ErrorCode::MissingData);
    assert_eq!(error.path(), "close[0]");
    assert!(error.to_string().contains("close[0]"));
    Ok(())
}

#[test]
fn expired_extremes_and_compensation_do_not_leak_between_windows() -> TestResult {
    let values = numbers(&[100.0, -50.0, 1.0, 1.0, 1.0])?;
    assert_eq!(ready(&rolling_min(&values, 3)?[4]), 1.0);
    assert_eq!(ready(&rolling_max(&values, 3)?[4]), 1.0);
    assert_eq!(ready(&rolling_sum(&values, 3)?[4]), 3.0);
    let values = numbers(&[1e16, 1.0, -1e16, 4.0, 4.0, 4.0])?;
    let output = rolling_sum(&values, 3)?;
    assert_eq!(ready(&output[2]), 1.0);
    assert_eq!(ready(&output[5]), 12.0);
    let zeros = numbers(&[-0.0, 0.0])?;
    for (_, operation) in OPERATIONS {
        assert_eq!(
            ready(&operation(&zeros, 2)?[1]).to_bits(),
            0.0_f64.to_bits()
        );
    }
    Ok(())
}

#[test]
fn numeric_overflow_is_distinct_from_nonfinite_input_and_is_atomic() -> TestResult {
    let huge = numbers(&[f64::MAX, f64::MAX])?;
    let before = huge.clone();
    for (name, operation) in OPERATIONS {
        if name == "min" || name == "max" {
            assert_eq!(ready(&operation(&huge, 2)?[1]), f64::MAX);
        } else {
            let error = operation(&huge, 2).err().ok_or("expected overflow")?;
            assert_eq!(error.code(), ErrorCode::NumericOverflow);
            assert_eq!(error.path(), format!("rolling.{name}[1]"));
            assert!(error.source().is_none());
        }
        assert_eq!(huge, before);
        assert_eq!(
            operation(&huge[..1], 2)?,
            vec![NumericValue::Warmup {
                required: 2,
                available: 1
            }]
        );
    }
    // Finite sum but overflowing subtraction; finite delta but overflowing square/square sum.
    for input in [
        vec![f64::MAX, -f64::MAX, -f64::MAX],
        vec![1e200, -1e200],
        vec![1e154, -1e154],
    ] {
        let input = numbers(&input)?;
        let before = input.clone();
        for operation in [rolling_variance as RollingFn, rolling_std] {
            assert_eq!(
                operation(&input, input.len()).map_err(|e| e.code()),
                Err(ErrorCode::NumericOverflow)
            );
            assert_eq!(input, before);
        }
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            FiniteF64::new(value).map_err(|e| e.code),
            Err(ErrorCode::NonFinite)
        );
    }
    assert!(serde_json::from_str::<Vec<FiniteF64>>("[null]").is_err());
    assert!(serde_json::from_str::<Vec<FiniteF64>>("[1e400]").is_err());
    Ok(())
}

#[test]
fn finite_underflow_is_explicit_and_single_large_sample_has_zero_variance() -> TestResult {
    let constant = numbers(&[0.1, 0.1, 0.1, 0.1])?;
    for operation in [rolling_variance as RollingFn, rolling_std] {
        for value in &operation(&constant, 3)?[2..] {
            assert_eq!(ready(value), 0.0);
        }
    }
    let input = numbers(&[0.0, f64::MIN_POSITIVE])?;
    assert_eq!(ready(&rolling_variance(&input, 2)?[1]), 0.0);
    assert_eq!(ready(&rolling_std(&input, 2)?[1]), 0.0);
    let input = numbers(&[f64::MAX, -f64::MAX])?;
    for operation in [rolling_variance as RollingFn, rolling_std] {
        assert!(operation(&input, 1)?
            .iter()
            .all(|value| ready(value) == 0.0));
    }
    Ok(())
}

fn timed_output(
    data: &MarketData,
    as_of: i64,
    operation: RollingFn,
    period: usize,
) -> Result<Vec<contracts::NumericSample>, Box<dyn Error>> {
    let available = AvailableData::new(data, as_of)?;
    let values = (0..available.len())
        .map(|i| {
            available
                .bar(i)
                .map(|bar| bar.close)
                .ok_or("missing available bar")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let output = operation(&values, period)?
        .into_iter()
        .enumerate()
        .map(|(index, value)| available.numeric_sample(index, value))
        .collect::<Result<Vec<_>, _>>()?;
    available.validate_numeric_series(&output)?;
    Ok(output)
}

#[test]
fn closed_bar_replay_and_changed_future_match_every_causal_prefix_exactly() -> TestResult {
    let data: MarketData = serde_json::from_str(include_str!("fixtures/temporal.json"))?;
    let original = serde_json::to_value(&data)?;
    let mut future = data.clone();
    for column in [
        &mut future.open,
        &mut future.high,
        &mut future.low,
        &mut future.close,
    ] {
        column[3] = Some(FiniteF64::new(1e200)?);
    }
    for as_of in [
        0, 999, 1000, 1999, 2000, 2499, 2500, 2999, 3000, 3499, 3500, 4000, 9000,
    ] {
        let len = data.available_len(as_of)?;
        let mut prefix = data.clone();
        prefix.timestamp.truncate(len);
        prefix.available_at.truncate(len);
        prefix.open.truncate(len);
        prefix.high.truncate(len);
        prefix.low.truncate(len);
        prefix.close.truncate(len);
        prefix.volume.truncate(len);
        prefix.is_closed.truncate(len);
        for period in [1, 2, 3, 14] {
            for (_, operation) in OPERATIONS {
                let expected = timed_output(&prefix, as_of, operation, period)?;
                assert_eq!(timed_output(&data, as_of, operation, period)?, expected);
                assert_eq!(timed_output(&future, as_of, operation, period)?, expected);
                assert!(expected.iter().all(|sample| sample.known_at <= as_of));
            }
        }
    }
    assert_eq!(serde_json::to_value(&data)?, original);
    Ok(())
}

#[test]
fn delayed_revision_requires_new_snapshot_and_retains_old_replay() -> TestResult {
    let old: MarketData = serde_json::from_str(include_str!("fixtures/temporal.json"))?;
    let mut revised = old.clone();
    revised.metadata.data_revision = "synthetic-r2".into();
    revised.metadata.revision_available_at = 6000;
    revised.metadata.snapshot_kind = contracts::SnapshotKind::RevisedHistory;
    revised.close[0] = Some(FiniteF64::new(2.0)?);
    assert_eq!(
        AvailableData::new(&revised, 5999).map_err(|e| e.code).err(),
        Some(ErrorCode::RevisionNotAvailable)
    );
    for (_, operation) in OPERATIONS {
        let old_output = timed_output(&old, 3500, operation, 2)?;
        let revised_output = timed_output(&revised, 6000, operation, 2)?;
        assert!(revised_output.iter().all(|sample| sample.known_at == 6000));
        assert_eq!(timed_output(&old, 3500, operation, 2)?, old_output);
    }
    let mut missing = old;
    missing.close[0] = None;
    assert_eq!(
        AvailableData::new(&missing, 3500).map_err(|e| e.code).err(),
        Some(ErrorCode::MissingData)
    );
    Ok(())
}

#[test]
fn changing_closed_future_bars_cannot_change_any_old_prefix() -> TestResult {
    let data: MarketData = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    for len in 0..=data.timestamp.len() {
        let as_of = if len == 0 { 0 } else { data.timestamp[len - 1] };
        let mut changed = data.clone();
        for column in [
            &mut changed.open,
            &mut changed.high,
            &mut changed.low,
            &mut changed.close,
        ] {
            for value in &mut column[len..] {
                *value = Some(FiniteF64::new(1e200)?);
            }
        }
        for (_, operation) in OPERATIONS {
            for period in [1, 2, 3, 14] {
                assert_eq!(
                    timed_output(&changed, as_of, operation, period)?,
                    timed_output(&data, as_of, operation, period)?
                );
            }
        }
    }
    Ok(())
}

#[test]
fn ramp_goldens_from_p0_remain_compatible() -> TestResult {
    let golden: Value = serde_json::from_str(include_str!("golden/p1-arithmetic.json"))?;
    let values = numbers(&[1.0, 2.0, 3.0])?;
    let aliases = BTreeMap::from([("mean", "sma")]);
    for (name, operation) in OPERATIONS {
        let name = aliases.get(name).copied().unwrap_or(name);
        let expected = golden["scalars_at_index_2"][name]
            .as_f64()
            .ok_or("missing P0 scalar")?;
        let actual = ready(&operation(&values, 3)?[2]);
        assert!((actual - expected).abs() <= 1e-12 + 1e-11 * expected.abs());
    }
    Ok(())
}
