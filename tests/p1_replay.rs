mod support;

use serde_json::{json, Value};
use std::{error::Error, fs};
use wyc_lib::{contracts::*, *};

type TestResult = Result<(), Box<dyn Error>>;

fn cases() -> Result<Vec<AnalysisRequest>, Box<dyn Error>> {
    let manifest: Value = serde_json::from_str(include_str!("fixtures/manifest.json"))?;
    let entry = manifest["fixtures"]
        .as_array()
        .ok_or("manifest")?
        .iter()
        .find(|f| f["fixture_id"] == "p1-structure-v1")
        .ok_or("structure manifest")?;
    support::verify_checksum(
        "tests/fixtures/p1-structure.json",
        include_bytes!("fixtures/p1-structure.json"),
        entry["files"][0]["sha256"].as_str().ok_or("sha256")?,
    )?;
    let fixture: Value = serde_json::from_str(include_str!("fixtures/p1-structure.json"))?;
    fixture["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .map(|c| {
            Ok(AnalysisRequest {
                schema_version: SCHEMA_VERSION.into(),
                data: serde_json::from_value(c["data"].clone())?,
                config: serde_json::from_value(c["config"].clone())?,
                as_of: c["as_of"].as_i64().ok_or("as_of")?,
                modules: vec![Module::Indicators, Module::MarketStructure],
            })
        })
        .collect()
}

fn append(state: &mut AnalysisState, data: &MarketData, i: usize, as_of: i64) -> TestResult {
    let update = ClosedBarUpdate {
        identity: state.identity().clone(),
        metadata: data.metadata.clone(),
        as_of,
        bar: ClosedBar {
            timestamp: data.timestamp[i],
            available_at: data.available_at[i],
            open: data.open[i],
            high: data.high[i],
            low: data.low[i],
            close: data.close[i],
            volume: data.volume[i],
            is_closed: data.is_closed[i],
        },
    };
    state.update_closed_bar(&update)?;
    Ok(())
}

#[test]
fn replay_matrix_keeps_old_as_of_and_emits_auditable_prefix_traces() -> TestResult {
    let mut traces = Vec::new();
    for (case, full) in cases()?.into_iter().enumerate() {
        let n = full.data.available_len(full.as_of)?;
        for grouped in [false, true] {
            for modules in [
                vec![Module::Indicators],
                vec![Module::MarketStructure],
                vec![Module::MarketStructure, Module::Indicators],
            ] {
                let mut full = full.clone();
                full.modules = modules;
                support::truncate(&mut full.data, n);
                if grouped {
                    // Two bars arrive together after closing. Calendar remains caller-owned.
                    for i in 0..n {
                        full.data.available_at[i] =
                            full.data.timestamp[(i / 2 * 2 + 1).min(n - 1)] + 777;
                    }
                }
                let mut prefix = full.clone();
                support::truncate(&mut prefix.data, 0);
                prefix.as_of = full.data.metadata.revision_available_at;
                let mut state = AnalysisState::new(&prefix)?;
                let mut saved = Vec::new();
                for i in 0..n {
                    let as_of =
                        full.data.available_at[i].max(full.data.metadata.revision_available_at);
                    append(&mut state, &full.data, i, as_of)?;
                    prefix = full.clone();
                    prefix.as_of = as_of;
                    support::truncate(&mut prefix.data, i + 1);
                    let before = prefix.clone();
                    let batch = analyze_batch(&prefix)?;
                    assert_eq!(state.result(), &batch, "case {case} prefix {}", i + 1);
                    assert_eq!(prefix, before);
                    if let Some(s) = &batch.structure {
                        assert!(s.evidence.iter().all(|e| e.known_at <= as_of));
                    }
                    let digest = support::checksum(&serde_json::to_vec(&batch)?);
                    // Full snapshots may reveal several bars at one as_of. Compare only
                    // once the entire availability group is appended.
                    let end_of_group = i + 1 == n || full.data.available_at[i + 1] > as_of;
                    if end_of_group {
                        let mut future = full.clone();
                        future.as_of = as_of;
                        // Change all future OHLCV while preserving input validity.
                        for j in i + 1..n {
                            future.data.high[j] = Some(FiniteF64::new(9999.0)?);
                            future.data.low[j] = Some(FiniteF64::new(-9999.0)?);
                            future.data.open[j] = Some(FiniteF64::new(0.0)?);
                            future.data.close[j] = Some(FiniteF64::new(0.0)?);
                            future.data.volume[j] = Some(FiniteF64::new(0.0)?);
                        }
                        assert_eq!(batch, analyze_batch(&future)?);
                        // Retain complete prior evaluations, including IDs and confirmation times.
                        saved.push((as_of, batch.clone()));
                    }
                    traces.push(json!({"case":case,"grouped_availability":grouped,
                        "modules":state.identity().modules,"prefix_len":i+1,"as_of":as_of,
                        "batch_state_equal":true,"future_checked":end_of_group,"sha256":digest}));
                }
                for (as_of, old) in saved {
                    let mut historical = full.clone();
                    historical.as_of = as_of;
                    assert_eq!(old, analyze_batch(&historical)?);
                }
            }
        }
    }
    assert!(!traces.is_empty());
    if let Some(path) = std::env::var_os("WYC_REPLAY_TRACE") {
        fs::write(
            path,
            serde_json::to_vec_pretty(&json!({
                "algorithm_version":ALGORITHM_VERSION,"fixture":"p1-structure-v1",
                "purpose":"parity and causality traces; not independent numeric goldens",
                "prefixes":traces.len(),"traces":traces
            }))?,
        )?;
    }
    Ok(())
}

#[test]
fn revised_snapshot_reset_then_append_matches_fresh_replay() -> TestResult {
    for mut full in cases()? {
        let n = full.data.available_len(full.as_of)?;
        support::truncate(&mut full.data, n);
        let mut state = AnalysisState::new(&full)?;
        let original = state.result().clone();
        let mut revised = full.clone();
        revised.data.metadata.data_revision.push_str("-g1-r2");
        revised.data.metadata.snapshot_kind = SnapshotKind::RevisedHistory;
        revised.as_of = full.as_of + 100_000;
        revised.data.metadata.revision_available_at = revised.as_of;
        // Rescale ALL history, maintaining exact OHLC ordering and changing all calculations.
        for column in [
            &mut revised.data.open,
            &mut revised.data.high,
            &mut revised.data.low,
            &mut revised.data.close,
        ] {
            for value in column {
                *value = value.map(|v| FiniteF64::new(v.get() * 2.0)).transpose()?;
            }
        }
        let mut half = revised.clone();
        support::truncate(&mut half.data, n / 2);
        assert_eq!(state.reset(&half)?, analyze_batch(&half)?);
        for i in n / 2..n {
            append(&mut state, &revised.data, i, revised.as_of)?;
        }
        assert_eq!(state.result(), &analyze_batch(&revised)?);
        assert_eq!(original, analyze_batch(&full)?);
        let before = state.clone();
        revised.as_of -= 1;
        assert_eq!(
            state.reset(&revised).err().ok_or("revision error")?.code(),
            ErrorCode::RevisionNotAvailable
        );
        assert_eq!(state, before);
    }
    Ok(())
}

#[test]
fn period_one_and_nondefault_first_valid_boundaries_through_public_api() -> TestResult {
    let data: MarketData = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    for period in [1, 2, 3, 8] {
        let config = AnalysisConfig {
            indicators: IndicatorConfig {
                period,
                macd_fast: 1,
                macd_slow: 2,
                macd_signal: 3,
                ..IndicatorConfig::default()
            },
            ..AnalysisConfig::default()
        };
        let result = calculate_indicators(&data, &config, 6000)?;
        for (kind, required) in [
            (IndicatorKind::Sma, period),
            (IndicatorKind::Ema, period),
            (IndicatorKind::Wma, period),
            (IndicatorKind::Atr, period),
            (IndicatorKind::Rsi, period + 1),
            (IndicatorKind::Adx, 2 * period),
            (IndicatorKind::MacdLine, 2),
            (IndicatorKind::MacdSignal, 4),
            (IndicatorKind::MacdHistogram, 4),
            (IndicatorKind::BollingerUpper, period),
        ] {
            for (i, sample) in result.output.series[&kind].iter().enumerate() {
                if i + 1 < required as usize {
                    assert_eq!(
                        sample.result,
                        NumericValue::Warmup {
                            required: required as usize,
                            available: i + 1
                        }
                    );
                } else {
                    assert!(matches!(sample.result, NumericValue::Ready { .. }));
                    // Independent closed forms for the integer ramp x_i=i+1.
                    let expected = match kind {
                        IndicatorKind::Sma | IndicatorKind::Ema => {
                            Some(i as f64 + 1.0 - (period as f64 - 1.0) / 2.0)
                        }
                        IndicatorKind::Wma => Some(i as f64 + 1.0 - (period as f64 - 1.0) / 3.0),
                        _ => None,
                    };
                    if let Some(value) = expected {
                        support::compare(
                            &json!(value),
                            &serde_json::to_value(&sample.result)?["value"],
                            1e-12,
                            1e-12,
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}
