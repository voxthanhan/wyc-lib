use std::error::Error;
use wyc_lib::{contracts::*, *};
type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn basic_state_uses_only_basic_cache_even_when_advanced_variance_overflows() -> TestResult {
    for scale in [1e-100, 1e200] {
        let mut full = request(80, scale)?;
        full.config.indicators.period = 7;
        let mut initial = full.clone();
        truncate(&mut initial.data, 0);
        initial.as_of = 0;
        let mut state = indicators::BasicIndicatorState::new(&initial.data, &initial.config, 0)?;
        for i in 0..80 {
            let mut at = full.clone();
            truncate(&mut at.data, i + 1);
            at.as_of = at.data.available_at[i];
            assert_eq!(
                state.update_closed_bar(&at.data, &at.config, at.as_of)?,
                indicators::basic::calculate_basic(&at.data, &at.config, at.as_of)?
            );
        }
        if scale > 1e100 {
            assert_eq!(
                calculate_indicators(&full.data, &full.config, full.as_of)
                    .err()
                    .ok_or("advanced variance overflow")?
                    .code(),
                ErrorCode::NumericOverflow
            );
        }
    }
    Ok(())
}

fn truncate(data: &mut MarketData, n: usize) {
    data.timestamp.truncate(n);
    data.available_at.truncate(n);
    data.is_closed.truncate(n);
    data.open.truncate(n);
    data.high.truncate(n);
    data.low.truncate(n);
    data.close.truncate(n);
    data.volume.truncate(n);
}
fn request(n: usize, scale: f64) -> Result<AnalysisRequest, Box<dyn Error>> {
    let mut data: MarketData = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    truncate(&mut data, 0);
    for i in 0..n {
        let phase = (i % 32) as f64;
        let price = (100.0 + phase.min(32.0 - phase) + (i / 32) as f64) * scale;
        data.timestamp.push((i + 1) as i64 * 1000);
        data.available_at
            .push(((i / 3) * 3 + 3) as i64 * 1000 + 200);
        data.open.push(Some(FiniteF64::new(price)?));
        data.close.push(Some(FiniteF64::new(price)?));
        data.high.push(Some(FiniteF64::new(price + scale)?));
        data.low.push(Some(FiniteF64::new(price - scale)?));
        data.volume
            .push(Some(FiniteF64::new((i % 13 * 100) as f64)?));
        data.is_closed.push(true);
    }
    Ok(AnalysisRequest {
        schema_version: SCHEMA_VERSION.into(),
        as_of: n as i64 * 1000 + 5000,
        data,
        modules: vec![Module::Indicators, Module::MarketStructure],
        config: AnalysisConfig::default(),
    })
}
fn update(state: &AnalysisState, r: &AnalysisRequest, i: usize, as_of: i64) -> ClosedBarUpdate {
    let d = &r.data;
    ClosedBarUpdate {
        identity: state.identity().clone(),
        metadata: d.metadata.clone(),
        as_of,
        bar: ClosedBar {
            timestamp: d.timestamp[i],
            available_at: d.available_at[i],
            open: d.open[i],
            high: d.high[i],
            low: d.low[i],
            close: d.close[i],
            volume: d.volume[i],
            is_closed: true,
        },
    }
}

#[test]
fn long_numeric_replay_is_exact_across_scales_and_seed_checkpoints() -> TestResult {
    for scale in [0.0, 1.0, 1e-100, 1e100] {
        for period in [1, 7, 32, 512] {
            let mut full = request(160, scale)?;
            full.modules = vec![Module::Indicators];
            full.config.indicators = IndicatorConfig {
                period,
                macd_fast: 2,
                macd_slow: 9,
                macd_signal: 4,
                ..IndicatorConfig::default()
            };
            let unchanged = full.clone();
            let mut prefix = full.clone();
            truncate(&mut prefix.data, 0);
            prefix.as_of = 0;
            let mut state = AnalysisState::new(&prefix)?;
            for i in 0..160 {
                let as_of = full.data.available_at[i];
                state.update_closed_bar(&update(&state, &full, i, as_of))?;
                prefix = full.clone();
                truncate(&mut prefix.data, i + 1);
                prefix.as_of = as_of;
                assert_eq!(
                    state.result(),
                    &analyze_batch(&prefix)?,
                    "scale {scale} period {period} index {i}"
                );
                if [0, 5, 31, 72].contains(&i) {
                    // Rebuild caches mid-seed and after seed, then keep appending.
                    state = AnalysisState::new(&prefix)?;
                }
            }
            assert_eq!(full, unchanged);
        }
    }
    Ok(())
}

#[test]
fn structure_failure_after_indicator_stage_rolls_back_all_caches_and_recovers() -> TestResult {
    let f: serde_json::Value = serde_json::from_str(include_str!("fixtures/p1-structure.json"))?;
    let mut r = request(9, 1.0)?;
    r.data = serde_json::from_value(f["cases"][0]["data"].clone())?;
    r.config = serde_json::from_value(f["cases"][0]["config"].clone())?;
    truncate(&mut r.data, 9);
    for (column, values) in [
        (
            &mut r.data.high,
            vec![6., 10., 7., 9., 8., 20., 21., 22., 23.],
        ),
        (
            &mut r.data.low,
            vec![4., 5., 4., 5., 6., 18., 17., 18., 11.],
        ),
        (
            &mut r.data.close,
            vec![5., 6., 5., 6., 7., 19., 19., 20., 12.],
        ),
    ] {
        *column = values
            .into_iter()
            .map(|v| FiniteF64::new(v).map(Some))
            .collect::<Result<_, _>>()?;
    }
    r.data.open = r.data.close.clone();
    r.as_of = 9200;
    let mut initial = r.clone();
    truncate(&mut initial.data, 8);
    initial.as_of = 8200;
    let mut state = AnalysisState::new(&initial)?;
    let before = state.clone();
    let held = state.result().clone();
    let batch_error = analyze_batch(&r).err().ok_or("batch conflict")?;
    let error = state
        .update_closed_bar(&update(&state, &r, 8, 9200))
        .err()
        .ok_or("state conflict")?;
    assert_eq!(error, batch_error);
    assert_eq!(state, before);
    r.data.close[8] = Some(FiniteF64::new(20.0)?);
    assert_eq!(
        state.update_closed_bar(&update(&state, &r, 8, 9200))?,
        analyze_batch(&r)?
    );
    assert_eq!(held, analyze_batch(&initial)?);
    Ok(())
}

#[test]
fn long_both_modules_reset_revision_then_append_keeps_event_order_and_ids() -> TestResult {
    let full = request(256, 1.0)?;
    let mut initial = full.clone();
    truncate(&mut initial.data, 97);
    initial.as_of = initial.data.available_at[96];
    let mut state = AnalysisState::new(&initial)?;
    for i in 97..256 {
        let mut prefix = full.clone();
        truncate(&mut prefix.data, i + 1);
        prefix.as_of = full.data.available_at[i];
        assert_eq!(
            state.update_closed_bar(&update(&state, &full, i, prefix.as_of))?,
            analyze_batch(&prefix)?
        );
    }
    let held = state.result().clone();
    let mut revised = full.clone();
    revised.as_of = 1_000_000;
    revised.data.metadata.data_revision = "incremental-r2".into();
    revised.data.metadata.revision_available_at = revised.as_of;
    revised.data.metadata.snapshot_kind = SnapshotKind::RevisedHistory;
    for column in [
        &mut revised.data.open,
        &mut revised.data.high,
        &mut revised.data.low,
        &mut revised.data.close,
    ] {
        for v in column {
            *v = v.map(|v| FiniteF64::new(v.get() * 0.5)).transpose()?;
        }
    }
    let mut half = revised.clone();
    truncate(&mut half.data, 97);
    assert_eq!(state.reset(&half)?, analyze_batch(&half)?);
    for i in 97..256 {
        let mut prefix = revised.clone();
        truncate(&mut prefix.data, i + 1);
        assert_eq!(
            state.update_closed_bar(&update(&state, &revised, i, revised.as_of))?,
            analyze_batch(&prefix)?
        );
    }
    let mut old = full;
    old.as_of = old.data.available_at[255];
    assert_eq!(held, analyze_batch(&old)?);
    Ok(())
}

#[test]
fn arithmetic_failure_near_seed_boundary_preserves_seed_then_recovers() -> TestResult {
    let mut r = request(12, 1.0)?;
    r.modules = vec![Module::Indicators];
    r.config.indicators.period = 7;
    let mut initial = r.clone();
    truncate(&mut initial.data, 6);
    initial.as_of = initial.data.available_at[5];
    let mut state = AnalysisState::new(&initial)?;
    let before = state.clone();
    let mut bad = update(&state, &r, 6, r.data.available_at[6]);
    bad.bar.high = Some(FiniteF64::new(1e308)?);
    bad.bar.low = Some(FiniteF64::new(-1e308)?);
    assert_eq!(
        state
            .update_closed_bar(&bad)
            .err()
            .ok_or("overflow")?
            .code(),
        ErrorCode::NumericOverflow
    );
    assert_eq!(state, before);
    for i in 6..12 {
        let mut prefix = r.clone();
        truncate(&mut prefix.data, i + 1);
        prefix.as_of = r.data.available_at[i];
        assert_eq!(
            state.update_closed_bar(&update(&state, &r, i, prefix.as_of))?,
            analyze_batch(&prefix)?
        );
    }
    Ok(())
}
