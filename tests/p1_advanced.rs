use contracts::{
    AnalysisConfig, AnalysisError, AvailableData, ContractError, ErrorCode, FiniteF64,
    IndicatorKind, MarketData, NumericValue,
};
use indicators::advanced::*;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;
type RawSeries = Vec<(&'static str, Vec<NumericValue>, usize)>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    fixture_id: String,
    algorithm_version: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    period: usize,
    fast: usize,
    slow: usize,
    signal: usize,
    multiplier: FiniteF64,
    close: Vec<FiniteF64>,
    high: Vec<FiniteF64>,
    low: Vec<FiniteF64>,
}
fn fixture() -> Result<Fixture, Box<dyn Error>> {
    Ok(serde_json::from_str(include_str!(
        "fixtures/p1-advanced.json"
    ))?)
}
fn numbers(x: &[f64]) -> Result<Vec<FiniteF64>, ContractError> {
    x.iter().copied().map(FiniteF64::new).collect()
}
fn number(x: &NumericValue) -> f64 {
    match x {
        NumericValue::Ready { value } => value.get(),
        x => panic!("not ready: {x:?}"),
    }
}
fn config(case: &Case) -> AnalysisConfig {
    let mut c = AnalysisConfig::default();
    c.indicators.period = case.period as u32;
    c.indicators.macd_fast = case.fast as u32;
    c.indicators.macd_slow = case.slow as u32;
    c.indicators.macd_signal = case.signal as u32;
    c.indicators.bollinger_multiplier = case.multiplier;
    c
}
fn market(case: &Case) -> Result<MarketData, Box<dyn Error>> {
    let mut d: MarketData = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    let n = case.close.len();
    d.timestamp = (1..=n).map(|i| i as i64 * 1000).collect();
    d.available_at = d.timestamp.iter().map(|t| t + 200).collect();
    d.open = case.close.iter().copied().map(Some).collect();
    d.close = d.open.clone();
    d.high = case.high.iter().copied().map(Some).collect();
    d.low = case.low.iter().copied().map(Some).collect();
    d.volume = vec![Some(FiniteF64::new(0.0)?); n];
    d.is_closed = vec![true; n];
    d.metadata.data_revision = "advanced-r1".into();
    Ok(d)
}
fn truncate(d: &mut MarketData, n: usize) {
    d.timestamp.truncate(n);
    d.available_at.truncate(n);
    d.open.truncate(n);
    d.close.truncate(n);
    d.high.truncate(n);
    d.low.truncate(n);
    d.volume.truncate(n);
    d.is_closed.truncate(n);
}
fn raw(c: &Case) -> Result<RawSeries, AnalysisError> {
    let m = macd(&c.close, c.fast, c.slow, c.signal)?;
    let a = adx(&c.high, &c.low, &c.close, c.period)?;
    let b = bollinger_bands(&c.close, c.period, c.multiplier)?;
    let q = c.slow + c.signal - 1;
    Ok(vec![
        ("rsi", rsi(&c.close, c.period)?, c.period + 1),
        ("macd_line", m.line, c.slow),
        ("macd_signal", m.signal, q),
        ("macd_histogram", m.histogram, q),
        ("plus_di", a.plus_di, c.period + 1),
        ("minus_di", a.minus_di, c.period + 1),
        ("dx", a.dx, c.period + 1),
        ("adx", a.adx, 2 * c.period),
        ("bollinger_middle", b.middle, c.period),
        ("bollinger_upper", b.upper, c.period),
        ("bollinger_lower", b.lower, c.period),
    ])
}
fn compare(
    actual: &[NumericValue],
    expected: &Value,
    required: usize,
    name: &str,
    tiny: bool,
    cancellation_budget: &[f64],
) -> TestResult {
    let expected = expected.as_array().ok_or("golden array")?;
    assert_eq!(actual.len(), expected.len());
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        a.validate()?;
        if e.is_null() {
            assert_eq!(
                *a,
                NumericValue::Warmup {
                    required,
                    available: i + 1
                },
                "{name}[{i}]"
            );
        } else {
            let (a, e) = (number(a), e.as_f64().ok_or("golden number")?);
            let percent = matches!(name, "rsi" | "plus_di" | "minus_di" | "dx" | "adx");
            let atol = if percent {
                1e-10
            } else if tiny {
                0.0
            } else {
                1e-12
            };
            assert!(
                (a - e).abs()
                    <= atol
                        + 1e-12 * a.abs().max(e.abs())
                        + cancellation_budget.get(i).copied().unwrap_or(0.0),
                "{name}[{i}]: {a} != {e}"
            );
        }
    }
    Ok(())
}

#[test]
fn independent_goldens_check_every_component_seed_and_sample() -> TestResult {
    let f = fixture()?;
    let g: Value = serde_json::from_str(include_str!("golden/p1-advanced.json"))?;
    assert_eq!(f.fixture_id, g["fixture_id"]);
    assert_eq!(f.algorithm_version, g["algorithm_version"]);
    let cases = g["cases"].as_array().ok_or("cases")?;
    assert_eq!(f.cases.len(), 8);
    assert_eq!(cases.len(), f.cases.len());
    for (c, e) in f.cases.iter().zip(cases) {
        assert_eq!(c.id, e["id"]);
        let d = market(c)?;
        let before = d.clone();
        let batch = calculate_advanced(&d, &config(c), 100_000)?;
        assert_eq!(batch.output.series.len(), 11);
        assert_eq!(batch.metadata.config_hash, config(c).config_hash()?);
        assert_eq!(batch.metadata.data_revision, "advanced-r1");
        for (name, values, required) in raw(c)? {
            // Mean +/- width can nearly cancel at huge input scales. The
            // existing exact-real golden stays unchanged; bound absolute
            // roundoff by eight machine epsilons of the input window scale.
            let budget: Vec<f64> = if c.id == "large" && name.starts_with("bollinger_") {
                (0..c.close.len())
                    .map(|i| {
                        8.0 * f64::EPSILON
                            * c.close[(i + 1).saturating_sub(c.period)..=i]
                                .iter()
                                .map(|v| v.get().abs())
                                .fold(0.0, f64::max)
                    })
                    .collect()
            } else {
                vec![]
            };
            compare(&values, &e[name], required, name, c.id == "tiny", &budget)?;
            let kind: IndicatorKind = serde_json::from_value(Value::String(name.into()))?;
            let samples = &batch.output.series[&kind];
            assert_eq!(
                values,
                samples.iter().map(|s| s.result.clone()).collect::<Vec<_>>()
            );
            AvailableData::new(&d, 100_000)?.validate_numeric_series(samples)?;
            for (i, s) in samples.iter().enumerate() {
                assert_eq!(s.observed_at, d.timestamp[i]);
                assert_eq!(s.known_at, d.available_at[i]);
            }
        }
        assert_eq!(d, before);
    }
    Ok(())
}

#[test]
fn original_ramp_scalars_agree_with_p0_hand_calculations() -> TestResult {
    let d: MarketData = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    let g: Value = serde_json::from_str(include_str!("golden/p1-arithmetic.json"))?;
    let c: AnalysisConfig = serde_json::from_value(g["config"].clone())?;
    let out = calculate_advanced(&d, &c, 6000)?;
    for (key, index) in [
        ("scalars_at_index_2", 2),
        ("scalars_at_index_3", 3),
        ("scalars_at_index_5", 5),
    ] {
        for (name, e) in g[key].as_object().ok_or("scalars")? {
            let kind: IndicatorKind = serde_json::from_value(Value::String(name.clone()))?;
            if let Some(s) = out.output.series.get(&kind) {
                assert!((number(&s[index].result) - e.as_f64().ok_or("scalar")?).abs() < 1e-10);
            }
        }
    }
    Ok(())
}

#[test]
fn every_prefix_replays_exactly_and_future_prices_cannot_change_past() -> TestResult {
    for c in fixture()?.cases {
        let d = market(&c)?;
        let cfg = config(&c);
        for n in 0..=d.timestamp.len() {
            let as_of = if n == 0 { 0 } else { d.available_at[n - 1] };
            let mut prefix = d.clone();
            truncate(&mut prefix, n);
            let expected = calculate_advanced(&prefix, &cfg, as_of)?;
            assert_eq!(calculate_advanced(&d, &cfg, as_of)?, expected);
            let mut changed = d.clone();
            for j in n..d.timestamp.len() {
                changed.close[j] = Some(FiniteF64::new(1e120)?);
                changed.open[j] = changed.close[j];
                changed.high[j] = changed.close[j];
                changed.low[j] = changed.close[j];
            }
            assert_eq!(calculate_advanced(&changed, &cfg, as_of)?, expected);
            assert_eq!(calculate_advanced(&d, &cfg, as_of)?, expected);
        }
    }
    Ok(())
}

#[test]
fn warmup_boundaries_cover_nondefault_and_maximum_periods() -> TestResult {
    let mut c = fixture()?.cases.remove(0);
    for n in [1, 2, 3, 14, 1_000_000] {
        c.period = n;
        for (f, s, q) in [
            (1, 2, 1),
            (2, 3, 4),
            (12, 26, 9),
            (999_999, 1_000_000, 1_000_000),
        ] {
            c.fast = f;
            c.slow = s;
            c.signal = q;
            for (name, values, required) in raw(&c)? {
                assert_eq!(values.len(), c.close.len());
                for (i, value) in values.iter().enumerate() {
                    if i + 1 < required {
                        assert_eq!(
                            *value,
                            NumericValue::Warmup {
                                required,
                                available: i + 1
                            },
                            "{name}"
                        );
                    } else {
                        assert!(matches!(value, NumericValue::Ready { .. }), "{name}[{i}]");
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn empty_and_single_bar_inputs_preserve_all_component_shapes() -> TestResult {
    for len in [0, 1] {
        let mut c = fixture()?.cases.remove(0);
        c.close.truncate(len);
        c.high.truncate(len);
        c.low.truncate(len);
        for (_, values, _) in raw(&c)? {
            assert_eq!(values.len(), len);
        }
        let out = calculate_advanced(&market(&c)?, &config(&c), 2000)?;
        assert_eq!(out.output.series.len(), 11);
        assert!(out.output.series.values().all(|s| s.len() == len));
    }
    Ok(())
}

#[test]
fn rsi_monotonic_and_flat_series_use_explicit_ready_conventions() -> TestResult {
    for n in [1, 2, 3, 14] {
        let up = numbers(&(0..40).map(|i| i as f64).collect::<Vec<_>>())?;
        let down = numbers(&(0..40).map(|i| -(i as f64)).collect::<Vec<_>>())?;
        let flat = numbers(&[0.1; 40])?;
        for (values, expected) in [(&up, 100.0), (&down, 0.0), (&flat, 50.0)] {
            assert!(rsi(values, n)?[n..].iter().all(|v| number(v) == expected));
        }
        let a = adx(&flat, &flat, &flat, n)?;
        assert!(a.adx[2 * n - 1..].iter().all(|v| number(v) == 0.0));
    }
    Ok(())
}

#[test]
fn dm_ties_zero_ranges_and_transition_seed_do_not_use_first_range() -> TestResult {
    let h = numbers(&[6.0, 7.0, 8.0])?;
    let l = numbers(&[4.0, 3.0, 2.0])?;
    let c = numbers(&[5.0; 3])?;
    let a = adx(&h, &l, &c, 1)?;
    for values in [a.plus_di, a.minus_di, a.dx, a.adx] {
        assert_eq!(number(&values[1]), 0.0);
        assert_eq!(number(&values[2]), 0.0);
    }
    let a = adx(
        &numbers(&[f64::MAX, 0.0])?,
        &numbers(&[-f64::MAX, 0.0])?,
        &numbers(&[0.0, 0.0])?,
        1,
    )?;
    assert_eq!(number(&a.adx[1]), 0.0);
    Ok(())
}

#[test]
fn invalid_configuration_is_rejected_even_for_empty_slices() -> TestResult {
    for n in [0, 1_000_001, usize::MAX] {
        for err in [
            rsi(&[], n).err(),
            macd(&[], n, 2, 1).err(),
            macd(&[], 1, n, 1).err(),
            macd(&[], 1, 2, n).err(),
            adx(&[], &[], &[], n).err(),
            bollinger_bands(&[], n, FiniteF64::new(2.0)?).err(),
        ] {
            assert_eq!(
                err.ok_or("expected error")?.code(),
                ErrorCode::InvalidConfig
            );
        }
    }
    for (f, s) in [(2, 2), (3, 2)] {
        assert_eq!(
            macd(&[], f, s, 1).err().ok_or("error")?.path(),
            "indicators.macd"
        );
    }
    for k in [0.0, -1.0] {
        assert_eq!(
            bollinger_bands(&[], 2, FiniteF64::new(k)?)
                .err()
                .ok_or("error")?
                .code(),
            ErrorCode::InvalidConfig
        );
    }
    for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(FiniteF64::new(v).is_err());
    }
    assert!(serde_json::from_str::<Vec<FiniteF64>>("[null]").is_err());
    Ok(())
}

#[test]
fn ohlc_shape_and_future_invalid_data_are_rejected_without_mutation() -> TestResult {
    let x = numbers(&[1.0, 2.0])?;
    for (h, l, c, code) in [
        (&x[..1], &x[..], &x[..], ErrorCode::LengthMismatch),
        (&x[..], &x[..1], &x[..], ErrorCode::LengthMismatch),
        (&x[..], &x[..], &x[1..], ErrorCode::LengthMismatch),
    ] {
        assert_eq!(adx(h, l, c, 2).err().ok_or("shape")?.code(), code);
    }
    assert_eq!(
        adx(&numbers(&[1.0])?, &numbers(&[2.0])?, &numbers(&[1.0])?, 1)
            .err()
            .ok_or("ohlc")?
            .code(),
        ErrorCode::InvalidOhlc
    );
    let mut d = market(&fixture()?.cases[0])?;
    let cfg = AnalysisConfig::default();
    d.close[10] = None;
    let before = d.clone();
    assert_eq!(
        calculate_advanced(&d, &cfg, 1200)
            .err()
            .ok_or("missing")?
            .code(),
        ErrorCode::MissingData
    );
    assert_eq!(d, before);
    Ok(())
}

#[test]
fn overflow_reports_component_paths_and_preserves_input() -> TestResult {
    let x = numbers(&[-f64::MAX, f64::MAX])?;
    let before = x.clone();
    let errors = [
        (rsi(&x, 1).err(), "indicators.rsi[1]"),
        (
            rsi(&numbers(&[0.0, -1e-300, 1e100])?, 2).err(),
            "indicators.rsi[2]",
        ),
        (
            macd(&numbers(&[f64::MAX, f64::MAX])?, 1, 2, 1).err(),
            "indicators.ema[1]",
        ),
        (
            adx(
                &numbers(&[0.0, 1e307])?,
                &numbers(&[0.0, 1e307])?,
                &numbers(&[0.0, 1e307])?,
                1,
            )
            .err(),
            "indicators.plus_di[1]",
        ),
        (
            bollinger_bands(&numbers(&[0.0, 3.0])?, 2, FiniteF64::new(f64::MAX)?).err(),
            "indicators.bollinger_width[1]",
        ),
        (
            bollinger_bands(&numbers(&[-1e200, 1e200])?, 2, FiniteF64::new(2.0)?).err(),
            "rolling.std[1]",
        ),
    ];
    for (err, path) in errors {
        let err = err.ok_or_else(|| format!("no overflow: {path}"))?;
        assert_eq!(err.code(), ErrorCode::NumericOverflow);
        assert_eq!(err.path(), path);
    }
    assert_eq!(x, before);
    Ok(())
}

#[test]
fn delayed_availability_open_bars_and_revision_have_explicit_lineage() -> TestResult {
    let c = fixture()?.cases.remove(0);
    let cfg = config(&c);
    let d = market(&c)?;
    let old = calculate_advanced(&d, &cfg, 12_200)?;
    let mut delayed = d.clone();
    delayed.available_at[11] = 13_000;
    let at_close = calculate_advanced(&delayed, &cfg, 12_200)?;
    assert!(at_close.output.series.values().all(|s| s.len() == 11));
    let at_available = calculate_advanced(&delayed, &cfg, 13_000)?;
    assert!(at_available
        .output
        .series
        .values()
        .all(|s| s[11].known_at == 13_000));
    delayed.is_closed[11] = false;
    assert!(calculate_advanced(&delayed, &cfg, 13_000)?
        .output
        .series
        .values()
        .all(|s| s.len() == 11));
    let mut revised = d.clone();
    revised.metadata.data_revision = "advanced-r2".into();
    revised.metadata.revision_available_at = 12_500;
    revised.close[2] = Some(FiniteF64::new(10.0)?);
    assert!(calculate_advanced(&revised, &cfg, 12_200).is_err());
    let new = calculate_advanced(&revised, &cfg, 12_500)?;
    assert_eq!(new.metadata.data_revision, "advanced-r2");
    assert_ne!(old.output, new.output);
    assert!(new
        .output
        .series
        .values()
        .flatten()
        .all(|s| s.known_at == 12_500));
    assert_eq!(calculate_advanced(&d, &cfg, 12_200)?, old);
    Ok(())
}

#[test]
fn underflow_is_finite_and_tiny_positive_denominators_are_not_missing() -> TestResult {
    let x = numbers(&[0.0, 1e-300, 2e-300, 1e-300, 3e-300])?;
    assert!(rsi(&x, 2)?[2..]
        .iter()
        .all(|v| matches!(v, NumericValue::Ready { .. })));
    let b = bollinger_bands(&x, 2, FiniteF64::new(2.0)?)?;
    // Squared deviations underflow according to IEEE-754; zero is a value.
    for i in 1..x.len() {
        assert_eq!(b.middle[i], b.upper[i]);
        assert_eq!(b.middle[i], b.lower[i]);
    }
    Ok(())
}
