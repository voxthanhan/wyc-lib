use contracts::{
    AnalysisConfig, AnalysisError, AvailableData, ContractError, DataGap, ErrorCode, FiniteF64,
    IndicatorKind, MarketData, MissingReason, NumericValue, SnapshotKind,
};
use indicators::{basic::*, BasicIndicatorState};
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;
type SeriesFn = fn(&[FiniteF64], usize) -> Result<Vec<NumericValue>, AnalysisError>;
const SERIES: [(&str, SeriesFn, usize); 7] = [
    ("sma", sma, 0),
    ("ema", ema, 0),
    ("wma", wma, 0),
    ("momentum", momentum, 1),
    ("roc", roc, 1),
    ("volume_sma", volume_sma, 0),
    ("rvol", rvol, 0),
];

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
    close: Vec<FiniteF64>,
    high: Vec<FiniteF64>,
    low: Vec<FiniteF64>,
    volume: Vec<FiniteF64>,
}
fn fixture() -> Result<Fixture, Box<dyn Error>> {
    Ok(serde_json::from_str(include_str!(
        "fixtures/p1-basic.json"
    ))?)
}
fn numbers(x: &[f64]) -> Result<Vec<FiniteF64>, ContractError> {
    x.iter().copied().map(FiniteF64::new).collect()
}
fn ready(x: &NumericValue) -> f64 {
    match x {
        NumericValue::Ready { value } => value.get(),
        x => panic!("not ready: {x:?}"),
    }
}
fn missing() -> NumericValue {
    NumericValue::Missing {
        reason: MissingReason::ZeroDenominator,
    }
}
fn config(period: usize) -> AnalysisConfig {
    let mut c = AnalysisConfig::default();
    c.indicators.period = period as u32;
    c
}
fn market(case: &Case) -> Result<MarketData, Box<dyn Error>> {
    let mut data: MarketData = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    let n = case.close.len();
    data.symbol = case.id.clone();
    data.timestamp = (1..=n).map(|i| i as i64 * 1000).collect();
    data.available_at = data.timestamp.clone();
    data.open = case.close.iter().copied().map(Some).collect();
    data.high = case.high.iter().copied().map(Some).collect();
    data.low = case.low.iter().copied().map(Some).collect();
    data.close = data.open.clone();
    data.volume = case.volume.iter().copied().map(Some).collect();
    data.is_closed = vec![true; n];
    data.metadata.data_revision = "basic-r1".into();
    Ok(data)
}
fn truncate(data: &mut MarketData, len: usize) {
    data.timestamp.truncate(len);
    data.available_at.truncate(len);
    data.open.truncate(len);
    data.high.truncate(len);
    data.low.truncate(len);
    data.close.truncate(len);
    data.volume.truncate(len);
    data.is_closed.truncate(len);
}
fn compare(
    actual: &[NumericValue],
    expected: &Value,
    period: usize,
    lag: usize,
    name: &str,
    tiny: bool,
) -> TestResult {
    let expected = expected.as_array().ok_or("golden array missing")?;
    assert_eq!(actual.len(), expected.len());
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        a.validate()?;
        if e.is_null() {
            assert_eq!(
                *a,
                NumericValue::Warmup {
                    required: period + lag,
                    available: i + 1
                }
            );
        } else if e.as_str() == Some("zero_denominator") {
            assert_eq!(*a, missing());
        } else {
            let e = e.as_f64().ok_or("non-numeric golden")?;
            let a = ready(a);
            let atol = if tiny {
                0.0
            } else if name == "roc" {
                1e-10
            } else {
                1e-12
            };
            assert!(
                (a - e).abs() <= atol + 1e-12 * a.abs().max(e.abs()),
                "{name}[{i}]: {a} != {e}"
            );
        }
    }
    Ok(())
}

#[test]
fn fraction_goldens_cover_every_sample_of_all_nine_indicators() -> TestResult {
    let f = fixture()?;
    let g: Value = serde_json::from_str(include_str!("golden/p1-basic.json"))?;
    assert_eq!(f.fixture_id, g["fixture_id"]);
    assert_eq!(f.algorithm_version, g["algorithm_version"]);
    assert_eq!(f.config["periods"], serde_json::json!([1, 2, 3]));
    let expected = g["cases"].as_array().ok_or("no golden cases")?;
    assert_eq!(f.cases.len(), expected.len());
    for (case, expected) in f.cases.iter().zip(expected) {
        assert_eq!(case.id, expected["id"]);
        let before = market(case)?;
        let output = calculate_basic(&before, &config(case.period), 9000)?;
        assert_eq!(output.output.series.len(), 9);
        for (name, op, lag) in SERIES {
            let input = if name == "volume_sma" || name == "rvol" {
                &case.volume
            } else {
                &case.close
            };
            compare(
                &op(input, case.period)?,
                &expected[name],
                case.period,
                lag,
                name,
                case.id == "tiny",
            )?;
        }
        compare(
            &true_range(&case.high, &case.low, &case.close)?,
            &expected["tr"],
            1,
            0,
            "tr",
            case.id == "tiny",
        )?;
        compare(
            &atr(&case.high, &case.low, &case.close, case.period)?,
            &expected["atr"],
            case.period,
            0,
            "atr",
            case.id == "tiny",
        )?;
        let kinds = [
            (IndicatorKind::Sma, "sma", case.period),
            (IndicatorKind::Ema, "ema", case.period),
            (IndicatorKind::Wma, "wma", case.period),
            (IndicatorKind::Tr, "tr", 1),
            (IndicatorKind::Atr, "atr", case.period),
            (IndicatorKind::Momentum, "momentum", case.period + 1),
            (IndicatorKind::Roc, "roc", case.period + 1),
            (IndicatorKind::VolumeSma, "volume_sma", case.period),
            (IndicatorKind::Rvol, "rvol", case.period),
        ];
        let available = AvailableData::new(&before, 9000)?;
        for (kind, name, required) in kinds {
            let samples = &output.output.series[&kind];
            available.validate_numeric_series(samples)?;
            let values: Vec<_> = samples.iter().map(|s| s.result.clone()).collect();
            compare(
                &values,
                &expected[name],
                required,
                0,
                name,
                case.id == "tiny",
            )?;
        }
        assert_eq!(market(case)?, before);
        assert_eq!(
            output.metadata.config_hash,
            config(case.period).config_hash()?
        );
        assert_eq!(output.metadata.schema_version, "0.1.0");
        assert_eq!(output.metadata.engine_version, "0.1.0");
        assert_eq!(output.metadata.algorithm_version, "p1-v1");
        assert_eq!(output.metadata.data_revision, before.metadata.data_revision);
    }
    Ok(())
}

#[test]
fn warmup_before_at_after_first_valid_and_empty_maximum_period() -> TestResult {
    for n in [1, 2, 3, 14, 1_000_000] {
        let len = if n == 1_000_000 { 3 } else { n + 2 };
        let x = numbers(&vec![2.0; len])?;
        for (name, op, lag) in SERIES {
            assert!(op(&[], n)?.is_empty());
            let output = op(&x, n)?;
            for (i, value) in output.iter().enumerate() {
                if i + 1 < n + lag {
                    assert_eq!(
                        *value,
                        NumericValue::Warmup {
                            required: n + lag,
                            available: i + 1
                        },
                        "{name}"
                    );
                } else {
                    assert!(matches!(value, NumericValue::Ready { .. }), "{name}");
                }
            }
        }
        assert!(atr(&[], &[], &[], n)?.is_empty());
        let ranges = atr(&x, &x, &x, n)?;
        for (i, value) in ranges.iter().enumerate() {
            if i + 1 < n {
                assert_eq!(
                    *value,
                    NumericValue::Warmup {
                        required: n,
                        available: i + 1
                    }
                );
            } else {
                assert_eq!(ready(value), 0.0);
            }
        }
    }
    assert!(true_range(&[], &[], &[])?.is_empty());
    Ok(())
}

#[test]
fn invalid_periods_shapes_ohlc_volume_and_nonfinite_are_rejected() -> TestResult {
    for n in [0, 1_000_001, usize::MAX] {
        for (_, op, _) in SERIES {
            let e = op(&[], n).err().ok_or("expected period error")?;
            assert_eq!(
                (e.code(), e.path()),
                (ErrorCode::InvalidConfig, "indicators.period")
            );
        }
        assert_eq!(
            atr(&[], &[], &[], n).map_err(|e| e.code()),
            Err(ErrorCode::InvalidConfig)
        );
    }
    let x = numbers(&[1.0, 2.0])?;
    assert_eq!(
        true_range(&[], &x, &x).map_err(|e| e.code()),
        Err(ErrorCode::LengthMismatch)
    );
    assert_eq!(
        atr(&x, &[], &x, 14).map_err(|e| e.code()),
        Err(ErrorCode::LengthMismatch)
    );
    assert_eq!(
        true_range(&numbers(&[0.0])?, &numbers(&[1.0])?, &numbers(&[0.5])?).map_err(|e| e.code()),
        Err(ErrorCode::InvalidOhlc)
    );
    assert_eq!(
        true_range(&x, &x, &numbers(&[0.0, 3.0])?).map_err(|e| e.code()),
        Err(ErrorCode::InvalidOhlc)
    );
    for op in [volume_sma as SeriesFn, rvol] {
        let negative = numbers(&[0.0, -1.0])?;
        let e = op(&negative, 14)
            .err()
            .ok_or("negative volume accepted during warmup")?;
        assert_eq!(
            (e.code(), e.path()),
            (ErrorCode::NegativeVolume, "indicators.volume[1]")
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            FiniteF64::new(value).map_err(|e| e.code),
            Err(ErrorCode::NonFinite)
        );
    }
    assert!(serde_json::from_str::<Vec<FiniteF64>>("[null]").is_err());
    Ok(())
}

#[test]
fn zero_denominators_recover_and_lag_does_not_use_current_mean() -> TestResult {
    let x = numbers(&[0.0, 0.0, 2.0, 4.0, 0.0, 3.0])?;
    let output = roc(&x, 2)?;
    assert_eq!(output[2], missing());
    assert_eq!(output[3], missing());
    assert_eq!(ready(&output[4]), -100.0);
    assert_eq!(ready(&output[5]), -25.0);
    let r = rvol(&x, 2)?;
    assert_eq!(r[1], missing());
    assert_eq!(ready(&r[2]), 2.0);
    assert_eq!(ready(&r[4]), 0.0);
    let x = numbers(&[1.0, 9.0, 3.0, 81.0])?;
    assert_eq!(ready(&momentum(&x, 2)?[3]), 72.0);
    assert_eq!(ready(&roc(&x, 2)?[3]), 800.0);
    Ok(())
}

#[test]
fn overflow_is_explicit_and_period_one_avoids_unnecessary_smoothing_subtraction() -> TestResult {
    let huge = numbers(&[f64::MAX, f64::MAX])?;
    for op in [sma as SeriesFn, ema, wma, volume_sma, rvol] {
        assert_eq!(
            op(&huge, 2).map_err(|e| e.code()),
            Err(ErrorCode::NumericOverflow)
        );
    }
    let x = numbers(&[f64::MAX, -f64::MAX, f64::MAX])?;
    assert_eq!(
        ema(&x, 1)?.iter().map(ready).collect::<Vec<_>>(),
        vec![f64::MAX, -f64::MAX, f64::MAX]
    );
    for (op, input, n, path) in [
        (
            ema as SeriesFn,
            vec![0.0, 0.0, f64::MAX, -f64::MAX],
            2,
            "indicators.ema[3]",
        ),
        (wma, vec![0.0, f64::MAX], 2, "indicators.wma[1]"),
        (
            momentum,
            vec![f64::MAX, -f64::MAX],
            1,
            "indicators.momentum[1]",
        ),
        (roc, vec![1e-300, f64::MAX], 1, "indicators.roc[1]"),
        (roc, vec![1.0, 1e307], 1, "indicators.roc[1]"),
    ] {
        let input = numbers(&input)?;
        let before = input.clone();
        let e = op(&input, n).err().ok_or("expected overflow")?;
        assert_eq!((e.code(), e.path()), (ErrorCode::NumericOverflow, path));
        assert_eq!(input, before);
    }
    let h = numbers(&[f64::MAX])?;
    let l = numbers(&[-f64::MAX])?;
    let c = numbers(&[0.0])?;
    let e = true_range(&h, &l, &c).err().ok_or("range overflow")?;
    assert_eq!(
        (e.code(), e.path()),
        (ErrorCode::NumericOverflow, "indicators.tr[0]")
    );
    let low = numbers(&[0.0, 0.0])?;
    assert!(atr(&huge, &low, &huge, 2).is_err());
    assert_eq!(
        atr(&huge, &low, &huge, 1)?
            .iter()
            .map(ready)
            .collect::<Vec<_>>(),
        vec![f64::MAX; 2]
    );
    Ok(())
}

#[test]
fn underflow_and_negative_zero_follow_f64_without_epsilon() -> TestResult {
    let subnormal = f64::from_bits(1);
    let x = numbers(&[subnormal, 0.0])?;
    assert_eq!(ready(&volume_sma(&x, 2)?[1]), 0.0);
    assert_eq!(rvol(&x, 2)?[1], missing());
    assert_eq!(
        ready(&roc(&numbers(&[f64::MAX, subnormal])?, 1)?[1]),
        -100.0
    );
    for (_, op, _) in SERIES {
        for value in op(&numbers(&[-0.0, 0.0, 0.0])?, 1)? {
            if let NumericValue::Ready { value } = value {
                assert_eq!(value.get().to_bits(), 0);
            }
        }
    }
    Ok(())
}

#[test]
fn every_fixture_prefix_has_exact_batch_state_parity() -> TestResult {
    for case in fixture()?.cases {
        let data = market(&case)?;
        let before = data.clone();
        for n in [1, 2, 3, 14, 1_000_000] {
            let c = config(n);
            let mut state = BasicIndicatorState::new(&data, &c, 0)?;
            for &as_of in &data.timestamp {
                let batch = calculate_basic(&data, &c, as_of)?;
                assert_eq!(state.update_closed_bar(&data, &c, as_of)?, batch);
                assert_eq!(*state.result(), batch);
            }
        }
        assert_eq!(data, before);
    }
    Ok(())
}

#[test]
fn availability_delays_future_changes_and_prefix_replay_are_causal() -> TestResult {
    let mut data = market(&fixture()?.cases[0])?;
    data.available_at = data.timestamp.iter().map(|t| t + 500).collect();
    data.is_closed[7] = false;
    let c = config(3);
    let mut state = BasicIndicatorState::new(&data, &c, 0)?;
    for len in 0..=7 {
        let as_of = if len == 0 {
            1000
        } else {
            data.available_at[len - 1]
        };
        let mut p = data.clone();
        truncate(&mut p, len);
        let mut future = data.clone();
        for column in [
            &mut future.open,
            &mut future.high,
            &mut future.low,
            &mut future.close,
            &mut future.volume,
        ] {
            for value in &mut column[len..] {
                *value = Some(FiniteF64::new(1e200)?);
            }
        }
        let expected = calculate_basic(&p, &c, as_of)?;
        assert_eq!(calculate_basic(&data, &c, as_of)?, expected);
        assert_eq!(calculate_basic(&future, &c, as_of)?, expected);
        if len > 0 {
            assert_eq!(state.update_closed_bar(&future, &c, as_of)?, expected);
        }
        for samples in expected.output.series.values() {
            assert!(samples.iter().all(|s| s.known_at <= as_of));
        }
        let at_close = if len == 0 { 0 } else { data.timestamp[len - 1] };
        assert_eq!(
            AvailableData::new(&data, at_close)?.len(),
            len.saturating_sub(1)
        );
    }
    let expected = state.result().clone();
    assert_eq!(calculate_basic(&data, &c, 99000)?.output, expected.output);
    Ok(())
}

#[test]
fn state_identity_changes_are_rejected_atomically() -> TestResult {
    let data = market(&fixture()?.cases[0])?;
    let c = config(3);
    let baseline = BasicIndicatorState::new(&data, &c, 3000)?;
    let mut variants = Vec::new();
    macro_rules! changed {
        ($field:ident, $value:expr) => {{
            let mut d = data.clone();
            d.$field = $value;
            variants.push(d);
        }};
    }
    changed!(symbol, "OTHER".into());
    changed!(timeframe, "OTHER".into());
    for field in [
        "source",
        "timezone",
        "price_unit",
        "volume_unit",
        "data_revision",
        "adjustment",
        "snapshot",
        "revision_time",
    ] {
        let mut d = data.clone();
        match field {
            "source" => d.metadata.source = "other".into(),
            "timezone" => d.metadata.timezone = "other".into(),
            "price_unit" => d.metadata.price_unit = "other".into(),
            "volume_unit" => d.metadata.volume_unit = "other".into(),
            "data_revision" => d.metadata.data_revision = "r2".into(),
            "adjustment" => d.metadata.adjustment_mode = contracts::AdjustmentMode::SplitAdjusted,
            "snapshot" => d.metadata.snapshot_kind = SnapshotKind::RevisedHistory,
            _ => d.metadata.revision_available_at = 1,
        }
        variants.push(d);
    }
    for variant in variants {
        let mut state = baseline.clone();
        assert_eq!(
            state
                .update_closed_bar(&variant, &c, 4000)
                .map_err(|e| e.code()),
            Err(ErrorCode::StateMismatch)
        );
        assert_eq!(state, baseline);
    }
    for c2 in [config(2), {
        let mut c = c.clone();
        c.structure.left += 1;
        c
    }] {
        let mut state = baseline.clone();
        assert_eq!(
            state
                .update_closed_bar(&data, &c2, 4000)
                .map_err(|e| e.code()),
            Err(ErrorCode::StateMismatch)
        );
        assert_eq!(state, baseline);
    }
    Ok(())
}

#[test]
fn duplicate_out_of_order_skipped_unavailable_gap_and_history_updates_are_atomic() -> TestResult {
    let data = market(&fixture()?.cases[0])?;
    let c = config(3);
    let baseline = BasicIndicatorState::new(&data, &c, 3000)?;
    for (as_of, expected) in [
        (2999, ErrorCode::InvalidAvailability),
        (3000, ErrorCode::InvalidAvailability),
        (5000, ErrorCode::ResetRequired),
    ] {
        let mut state = baseline.clone();
        assert_eq!(
            state
                .update_closed_bar(&data, &c, as_of)
                .map_err(|e| e.code()),
            Err(expected)
        );
        assert_eq!(state, baseline);
    }
    let mut variants = Vec::new();
    let mut d = data.clone();
    d.timestamp[3] = 3000;
    variants.push((d, ErrorCode::DuplicateTimestamp));
    let mut d = data.clone();
    d.timestamp[3] = 2999;
    variants.push((d, ErrorCode::UnorderedTimestamp));
    let mut d = data.clone();
    d.close[0] = Some(FiniteF64::new(11.0)?);
    variants.push((d, ErrorCode::ResetRequired));
    let mut d = data.clone();
    d.available_at[0] = 1500;
    variants.push((d, ErrorCode::ResetRequired));
    let mut d = data.clone();
    truncate(&mut d, 2);
    variants.push((d, ErrorCode::ResetRequired));
    let mut d = data.clone();
    d.metadata.gaps.push(DataGap {
        after_timestamp: 3000,
        before_timestamp: 4000,
    });
    variants.push((d, ErrorCode::ResetRequired));
    let mut d = data.clone();
    d.available_at[3] = 4500;
    variants.push((d, ErrorCode::InvalidAvailability));
    let mut d = data.clone();
    d.is_closed[3..].fill(false);
    variants.push((d, ErrorCode::InvalidAvailability));
    let mut d = data.clone();
    d.close[3] = None;
    variants.push((d, ErrorCode::MissingData));
    for (variant, code) in variants {
        let mut state = baseline.clone();
        assert_eq!(
            state
                .update_closed_bar(&variant, &c, 4000)
                .map_err(|e| e.code()),
            Err(code)
        );
        assert_eq!(state, baseline);
    }
    // Timestamp gaps alone do not imply missing bars without a caller calendar.
    let mut shifted = data.clone();
    for t in &mut shifted.timestamp[3..] {
        *t += 10000;
    }
    shifted.available_at = shifted.timestamp.clone();
    let mut state = baseline.clone();
    state.update_closed_bar(&shifted, &c, 14000)?;
    Ok(())
}

#[test]
fn overflow_update_and_invalid_reset_do_not_destroy_state_or_prior_outputs() -> TestResult {
    let data = market(&fixture()?.cases[0])?;
    let c = config(2);
    let mut state = BasicIndicatorState::new(&data, &c, 3000)?;
    let before = state.clone();
    let prior = state.result().clone();
    let mut huge = data.clone();
    for column in [
        &mut huge.open,
        &mut huge.high,
        &mut huge.low,
        &mut huge.close,
    ] {
        column[3] = Some(FiniteF64::new(f64::MAX)?);
    }
    assert_eq!(
        state
            .update_closed_bar(&huge, &c, 4000)
            .map_err(|e| e.code()),
        Err(ErrorCode::NumericOverflow)
    );
    assert_eq!(state, before);
    let mut invalid = data.clone();
    invalid.close[0] = None;
    assert_eq!(
        state.reset(&invalid, &c, 5000).map_err(|e| e.code()),
        Err(ErrorCode::MissingData)
    );
    assert_eq!(state, before);
    let mut invalid_config = c.clone();
    invalid_config.indicators.period = 0;
    assert_eq!(
        state
            .reset(&data, &invalid_config, 4000)
            .map_err(|e| e.code()),
        Err(ErrorCode::InvalidConfig)
    );
    assert_eq!(state, before);
    state.update_closed_bar(&data, &c, 4000)?;
    assert_eq!(prior, *before.result());
    Ok(())
}

#[test]
fn revision_reset_matches_new_batch_and_preserves_old_revision_replay() -> TestResult {
    let old = market(&fixture()?.cases[0])?;
    let c = config(3);
    let mut state = BasicIndicatorState::new(&old, &c, 3000)?;
    let prior = state.result().clone();
    let mut revised = old.clone();
    revised.close[0] = Some(FiniteF64::new(11.0)?);
    revised.metadata.data_revision = "r2".into();
    revised.metadata.revision_available_at = 4500;
    revised.metadata.snapshot_kind = SnapshotKind::RevisedHistory;
    assert_eq!(
        state.reset(&revised, &c, 4499).map_err(|e| e.code()),
        Err(ErrorCode::RevisionNotAvailable)
    );
    assert_eq!(*state.result(), prior);
    assert_eq!(
        state.reset(&revised, &c, 4500)?,
        calculate_basic(&revised, &c, 4500)?
    );
    assert!(state
        .result()
        .output
        .series
        .values()
        .flatten()
        .all(|s| s.known_at == 4500));
    assert_eq!(
        state.update_closed_bar(&revised, &c, 5000)?,
        calculate_basic(&revised, &c, 5000)?
    );
    assert_eq!(calculate_basic(&old, &c, 3000)?, prior);
    assert_eq!(
        state.reset(&old, &config(1), 2000)?,
        calculate_basic(&old, &config(1), 2000)?
    );
    Ok(())
}

#[test]
fn market_api_validates_unavailable_tail_and_never_repairs_input() -> TestResult {
    let data = market(&fixture()?.cases[0])?;
    let c = config(3);
    let empty = calculate_basic(&data, &c, 0)?;
    assert_eq!(empty.output.series.len(), 9);
    assert!(empty.output.series.values().all(Vec::is_empty));
    let mut variants = Vec::new();
    let mut d = data.clone();
    d.close[7] = None;
    variants.push((d, ErrorCode::MissingData));
    let mut d = data.clone();
    d.volume[7] = Some(FiniteF64::new(-1.0)?);
    variants.push((d, ErrorCode::NegativeVolume));
    let mut d = data.clone();
    d.schema_version = "future".into();
    variants.push((d, ErrorCode::UnsupportedVersion));
    let mut d = data.clone();
    d.metadata.revision_available_at = 9000;
    variants.push((d, ErrorCode::RevisionNotAvailable));
    let mut d = data.clone();
    d.high[7] = Some(FiniteF64::new(0.0)?);
    variants.push((d, ErrorCode::InvalidOhlc));
    for (variant, code) in variants {
        let before = variant.clone();
        assert_eq!(
            calculate_basic(&variant, &c, 3000).map_err(|e| e.code()),
            Err(code)
        );
        assert_eq!(variant, before);
    }
    let mut bad_config = c.clone();
    bad_config.indicators.period = 0;
    assert_eq!(
        calculate_basic(&data, &bad_config, 0).map_err(|e| e.code()),
        Err(ErrorCode::InvalidConfig)
    );
    Ok(())
}

#[test]
fn original_p0_scalar_goldens_are_still_compatible() -> TestResult {
    let data: MarketData = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    let golden: Value = serde_json::from_str(include_str!("golden/p1-arithmetic.json"))?;
    let result = calculate_basic(&data, &config(3), 6000)?;
    for (i, key) in [(2, "scalars_at_index_2"), (3, "scalars_at_index_3")] {
        for (kind, name) in [
            (IndicatorKind::Sma, "sma"),
            (IndicatorKind::Ema, "ema"),
            (IndicatorKind::Wma, "wma"),
            (IndicatorKind::Tr, "tr"),
            (IndicatorKind::Atr, "atr"),
            (IndicatorKind::Momentum, "momentum"),
            (IndicatorKind::Roc, "roc"),
            (IndicatorKind::VolumeSma, "volume_sma"),
            (IndicatorKind::Rvol, "rvol"),
        ] {
            if let Some(e) = golden[key][name].as_f64() {
                assert!((ready(&result.output.series[&kind][i].result) - e).abs() <= 1e-12);
            }
        }
    }
    Ok(())
}
