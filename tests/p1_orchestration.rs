mod support;

use serde_json::{json, Value};
use std::{error::Error, fs, path::PathBuf};
use wyc_lib::{contracts::*, *};

type TestResult = Result<(), Box<dyn Error>>;

fn request() -> Result<AnalysisRequest, Box<dyn Error>> {
    let data = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    let g: Value = serde_json::from_str(include_str!("golden/p1-arithmetic.json"))?;
    Ok(AnalysisRequest {
        schema_version: SCHEMA_VERSION.into(),
        data,
        as_of: 6000,
        modules: vec![Module::Indicators, Module::MarketStructure],
        config: serde_json::from_value(g["config"].clone())?,
    })
}

fn update(state: &AnalysisState, data: &MarketData, i: usize, as_of: i64) -> ClosedBarUpdate {
    ClosedBarUpdate {
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
    }
}

fn error<T: std::fmt::Debug>(result: Result<T, AnalysisError>, code: ErrorCode, path: &str) {
    let e = match result {
        Err(e) => e,
        Ok(x) => panic!("expected error, got {x:?}"),
    };
    assert_eq!(e.code(), code);
    assert_eq!(e.path(), path);
}

#[test]
fn complete_catalog_matches_hand_calculated_arithmetic_and_wire_shape() -> TestResult {
    let r = request()?;
    let before = r.clone();
    let result = analyze_batch(&r)?;
    let standalone = calculate_indicators(&r.data, &r.config, r.as_of)?;
    assert_eq!(result.indicators.as_ref(), Some(&standalone.output));
    assert_eq!(result.metadata, standalone.metadata);
    let structure = analyze_structure(&r.data, &r.config, r.as_of)?;
    assert_eq!(result.structure.as_ref(), Some(&structure.output));
    assert_eq!(result.metadata, structure.metadata);
    let wire = serde_json::to_value(&result)?;
    let series = wire["indicators"]["series"].as_object().ok_or("series")?;
    assert_eq!(series.len(), 25);
    let golden: Value = serde_json::from_str(include_str!("golden/p1-arithmetic.json"))?;
    for index in [2, 3, 5] {
        for (name, value) in golden[format!("scalars_at_index_{index}")]
            .as_object()
            .ok_or("scalars")?
        {
            support::compare(value, &series[name][index]["result"]["value"], 1e-12, 1e-12)?;
        }
    }
    assert_eq!(result, serde_json::from_value(wire)?);
    assert_eq!(r, before);
    assert_eq!(result.metadata.config_hash, r.config.config_hash()?);
    assert_eq!(result.metadata.engine_version, ENGINE_VERSION);
    assert_eq!(result.metadata.algorithm_version, ALGORITHM_VERSION);
    Ok(())
}

// Read the independent, immutable P1.3/P1.4 goldens through the NEW entry point.
// Expected numbers are never computed by the engine under test.
#[test]
fn basic_and_advanced_independent_goldens_through_full_catalog() -> TestResult {
    for (fixture, golden) in [
        (
            include_str!("fixtures/p1-basic.json"),
            include_str!("golden/p1-basic.json"),
        ),
        (
            include_str!("fixtures/p1-advanced.json"),
            include_str!("golden/p1-advanced.json"),
        ),
    ] {
        let f: Value = serde_json::from_str(fixture)?;
        let g: Value = serde_json::from_str(golden)?;
        for (c, expected) in f["cases"]
            .as_array()
            .ok_or("cases")?
            .iter()
            .zip(g["cases"].as_array().ok_or("goldens")?)
        {
            let mut r = request()?;
            let n = c["close"].as_array().ok_or("close")?.len();
            r.data.timestamp = (1..=n).map(|i| i as i64 * 1000).collect();
            r.data.available_at = r.data.timestamp.iter().map(|t| t + 200).collect();
            for (column, name) in [
                (&mut r.data.close, "close"),
                (&mut r.data.high, "high"),
                (&mut r.data.low, "low"),
            ] {
                *column = serde_json::from_value(c[name].clone())?;
            }
            r.data.open = r.data.close.clone();
            r.data.volume = if c.get("volume").is_some() {
                serde_json::from_value(c["volume"].clone())?
            } else {
                vec![Some(FiniteF64::new(0.0)?); n]
            };
            r.data.is_closed = vec![true; n];
            r.as_of = 100_000;
            r.config.indicators.period = c["period"].as_u64().ok_or("period")? as u32;
            if c.get("fast").is_some() {
                r.config.indicators.macd_fast = c["fast"].as_u64().ok_or("fast")? as u32;
                r.config.indicators.macd_slow = c["slow"].as_u64().ok_or("slow")? as u32;
                r.config.indicators.macd_signal = c["signal"].as_u64().ok_or("signal")? as u32;
                r.config.indicators.bollinger_multiplier =
                    serde_json::from_value(c["multiplier"].clone())?;
            }
            let result = calculate_indicators(&r.data, &r.config, r.as_of)?;
            let mut initial = r.clone();
            initial.modules = vec![Module::Indicators];
            initial.as_of = 0;
            let mut state = AnalysisState::new(&initial)?;
            for i in 0..n {
                let as_of = r.data.available_at[i];
                let actual = state.update_closed_bar(&update(&state, &r.data, i, as_of))?;
                let mut at = initial.clone();
                at.as_of = as_of;
                assert_eq!(actual, analyze_batch(&at)?);
            }
            let wire = serde_json::to_value(&result.output)?;
            assert_eq!(expected["id"], c["id"]);
            for (name, values) in expected.as_object().ok_or("case")? {
                if name == "id" {
                    continue;
                }
                for (i, e) in values.as_array().ok_or("values")?.iter().enumerate() {
                    let a = &wire["series"][name][i]["result"];
                    if e.is_null() {
                        assert_eq!(a["status"], "warmup");
                    } else if e.is_string() {
                        assert_eq!(a, &json!({"status":"missing","reason":e}));
                    } else {
                        assert_eq!(a["status"], "ready");
                        let percent = matches!(
                            name.as_str(),
                            "roc" | "rsi" | "plus_di" | "minus_di" | "dx" | "adx"
                        );
                        let mut atol = if percent {
                            1e-10
                        } else if c["id"] == "tiny" {
                            0.0
                        } else {
                            1e-12
                        };
                        if c["id"] == "large" && name.starts_with("bollinger_") {
                            let p = r.config.indicators.period as usize;
                            let scale = r.data.close[(i + 1).saturating_sub(p)..=i]
                                .iter()
                                .filter_map(|v| v.map(|v| v.get().abs()))
                                .fold(0.0, f64::max);
                            atol += 8.0 * f64::EPSILON * scale; // existing P1.4 budget
                        }
                        support::compare(e, &a["value"], atol, 1e-12)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn structure_goldens_and_all_prefixes_match_state_batch_and_old_as_of() -> TestResult {
    let f: Value = serde_json::from_str(include_str!("fixtures/p1-structure.json"))?;
    let g: Value = serde_json::from_str(include_str!("golden/p1-structure.json"))?;
    for (c, expected) in f["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .zip(g["cases"].as_array().ok_or("goldens")?)
    {
        let mut full = request()?;
        full.data = serde_json::from_value(c["data"].clone())?;
        full.config = serde_json::from_value(c["config"].clone())?;
        full.as_of = c["as_of"].as_i64().ok_or("as_of")?;
        full.modules = vec![Module::MarketStructure];
        support::compare(
            &expected["result"],
            &serde_json::to_value(analyze_structure(&full.data, &full.config, full.as_of)?)?,
            1e-12,
            1e-12,
        )?;
        for modules in [
            vec![Module::MarketStructure],
            vec![Module::Indicators],
            vec![Module::MarketStructure, Module::Indicators],
        ] {
            full.modules = modules;
            let n = full.data.available_len(full.as_of)?;
            let mut prefix = full.clone();
            support::truncate(&mut prefix.data, 0);
            prefix.as_of = full.data.metadata.revision_available_at;
            let mut state = AnalysisState::new(&prefix)?;
            for i in 0..n {
                let as_of = full.data.available_at[i].max(full.data.metadata.revision_available_at);
                let u = update(&state, &full.data, i, as_of);
                let actual = state.update_closed_bar(&u)?;
                let mut at = full.clone();
                at.as_of = as_of;
                // Equal availability may reveal multiple bars in the full snapshot;
                // the append protocol compares to the explicitly supplied prefix.
                support::truncate(&mut at.data, i + 1);
                assert_eq!(actual, analyze_batch(&at)?, "{} prefix {i}", c["id"]);
                assert_eq!(state.history_len(), i + 1);
                let prior = analyze_batch(&prefix)?;
                let mut future = full.clone();
                future.as_of = prefix.as_of;
                if future.data.available_len(future.as_of)? == prefix.data.timestamp.len() {
                    assert_eq!(prior, analyze_batch(&future)?);
                }
                prefix = at;
            }
        }
    }
    Ok(())
}

#[test]
fn replay_both_modules_and_kept_reports_are_immutable() -> TestResult {
    let full = request()?;
    let mut start = full.clone();
    start.as_of = 0;
    let mut state = AnalysisState::new(&start)?;
    let historical = state.result().clone();
    for i in 0..full.data.timestamp.len() {
        let actual =
            state.update_closed_bar(&update(&state, &full.data, i, full.data.available_at[i]))?;
        let mut at = full.clone();
        at.as_of = full.data.available_at[i];
        assert_eq!(actual, analyze_batch(&at)?);
        assert_eq!(
            state.identity().last_timestamp,
            Some(full.data.timestamp[i])
        );
    }
    assert_eq!(historical, analyze_batch(&start)?);
    Ok(())
}

fn conflict() -> Result<AnalysisRequest, Box<dyn Error>> {
    let mut r = request()?;
    let f: Value = serde_json::from_str(include_str!("fixtures/p1-structure.json"))?;
    r.data = serde_json::from_value(f["cases"][0]["data"].clone())?;
    r.config = serde_json::from_value(f["cases"][0]["config"].clone())?;
    support::truncate(&mut r.data, 9);
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
    Ok(r)
}

#[test]
fn selection_proves_unrequested_engines_do_not_execute() -> TestResult {
    let mut r = conflict()?;
    r.modules = vec![Module::Indicators];
    let output = analyze_batch(&r)?;
    assert!(output.structure.is_none());
    assert_eq!(serde_json::to_value(&output)?["structure"], Value::Null);
    r.modules.push(Module::MarketStructure);
    error(
        analyze_batch(&r),
        ErrorCode::InvalidResult,
        "structure.breaks[8].references",
    );
    let mut r = request()?;
    for column in [
        &mut r.data.open,
        &mut r.data.high,
        &mut r.data.low,
        &mut r.data.close,
    ] {
        for value in column.iter_mut() {
            *value = Some(FiniteF64::new(1e308)?);
        }
    }
    r.modules = vec![Module::MarketStructure];
    let output = analyze_batch(&r)?;
    assert!(output.indicators.is_none());
    assert_eq!(serde_json::to_value(output)?["indicators"], Value::Null);
    r.modules.push(Module::Indicators);
    assert_eq!(
        analyze_batch(&r).err().ok_or("overflow")?.code(),
        ErrorCode::NumericOverflow
    );
    Ok(())
}

#[test]
fn normalized_module_order_and_request_errors() -> TestResult {
    let r = request()?;
    let mut reversed = r.clone();
    reversed.modules.reverse();
    assert_eq!(analyze_batch(&r)?, analyze_batch(&reversed)?);
    assert_eq!(AnalysisState::new(&r)?, AnalysisState::new(&reversed)?);
    for (modules, code) in [
        (vec![], ErrorCode::InvalidConfig),
        (
            vec![Module::Indicators, Module::Indicators],
            ErrorCode::DuplicateModule,
        ),
        (vec![Module::PriceZones], ErrorCode::UnsupportedModule),
    ] {
        let mut bad = r.clone();
        bad.modules = modules;
        error(analyze_batch(&bad), code, "modules");
    }
    let mut bad = r.clone();
    bad.config.indicators.period = 0;
    error(
        analyze_batch(&bad),
        ErrorCode::InvalidConfig,
        "config.indicators.period",
    );
    bad = r.clone();
    bad.data.close[5] = None;
    bad.as_of = 0;
    error(analyze_batch(&bad), ErrorCode::MissingData, "close[5]");
    Ok(())
}

#[test]
fn empty_short_and_long_warmup_have_complete_containers() -> TestResult {
    for modules in [
        vec![Module::Indicators],
        vec![Module::MarketStructure],
        vec![Module::MarketStructure, Module::Indicators],
    ] {
        for n in [0, 1, 2] {
            let mut r = request()?;
            r.modules = modules.clone();
            support::truncate(&mut r.data, n);
            r.config.indicators.period = 1_000_000;
            let result = analyze_batch(&r)?;
            if let Some(indicators) = result.indicators {
                assert_eq!(indicators.series.len(), 25);
                for s in indicators.series.values() {
                    assert_eq!(s.len(), n);
                }
            }
            if let Some(structure) = result.structure {
                assert_eq!(structure.trends.len(), n);
                assert!(structure.pivots.is_empty());
            }
        }
    }
    Ok(())
}

#[test]
fn delayed_open_and_changed_future_are_never_retained() -> TestResult {
    let mut r = request()?;
    r.as_of = 3000;
    r.data.available_at[2] = 3500;
    let mut state = AnalysisState::new(&r)?;
    assert_eq!(state.history_len(), 2);
    let old = state.result().clone();
    // Send a corrected future bar; it was never part of retained history.
    r.data.close[2] = Some(FiniteF64::new(3.5)?);
    assert_eq!(old, analyze_batch(&r)?);
    let mut u = update(&state, &r.data, 2, 3499);
    error(
        state.update_closed_bar(&u),
        ErrorCode::InvalidAvailability,
        "state.new_bar",
    );
    u.as_of = 3500;
    let actual = state.update_closed_bar(&u)?;
    r.as_of = 3500;
    assert_eq!(actual, analyze_batch(&r)?);
    r.data.is_closed[3..].fill(false);
    r.as_of = 10000;
    let state = AnalysisState::new(&r)?;
    assert_eq!(state.history_len(), 3);
    assert_eq!(
        state
            .result()
            .indicators
            .as_ref()
            .ok_or("indicators")?
            .series[&IndicatorKind::Sma]
            .len(),
        3
    );
    Ok(())
}

#[test]
fn every_identity_and_metadata_change_is_atomic() -> TestResult {
    let mut r = request()?;
    r.as_of = 2000;
    let mut state = AnalysisState::new(&r)?;
    let original = state.clone();
    let good = update(&state, &r.data, 2, 3000);
    let wire = serde_json::to_value(&good)?;
    for (field, replacement) in [
        ("schema_version", json!("new")),
        ("engine_version", json!("new")),
        ("algorithm_version", json!("new")),
        ("symbol", json!("other")),
        ("timeframe", json!("other")),
        ("data_revision", json!("r2")),
        ("config_hash", json!("changed")),
        ("modules", json!(["indicators"])),
        ("last_timestamp", json!(null)),
        ("last_as_of", json!(0)),
    ] {
        let mut changed = wire.clone();
        changed["identity"][field] = replacement;
        let changed: ClosedBarUpdate = serde_json::from_value(changed)?;
        error(
            state.update_closed_bar(&changed),
            ErrorCode::StateMismatch,
            "state.identity",
        );
        assert_eq!(state, original);
    }
    for (field, replacement) in [
        ("source", json!("other")),
        ("timezone", json!("other")),
        ("price_unit", json!("other")),
        ("volume_unit", json!("other")),
        ("adjustment_mode", json!("split_adjusted")),
        ("data_revision", json!("r2")),
        ("revision_available_at", json!(1)),
        ("snapshot_kind", json!("revised_history")),
    ] {
        let mut changed = wire.clone();
        changed["metadata"][field] = replacement;
        error(
            state.update_closed_bar(&serde_json::from_value(changed)?),
            ErrorCode::StateMismatch,
            "state.identity",
        );
        assert_eq!(state, original);
    }
    state.update_closed_bar(&good)?;
    let advanced = state.clone();
    error(
        state.update_closed_bar(&good),
        ErrorCode::StateMismatch,
        "state.identity",
    );
    assert_eq!(state, advanced);
    Ok(())
}

#[test]
fn invalid_bars_and_as_of_preserve_exact_state() -> TestResult {
    let mut r = request()?;
    r.as_of = 2000;
    let mut state = AnalysisState::new(&r)?;
    let original = state.clone();
    let good = update(&state, &r.data, 2, 3000);
    let mut trials = Vec::new();
    let mut u = good.clone();
    u.bar.timestamp = 2000;
    trials.push((u, ErrorCode::DuplicateTimestamp, "timestamp[2]"));
    let mut u = good.clone();
    u.bar.timestamp = 1000;
    trials.push((u, ErrorCode::UnorderedTimestamp, "timestamp[2]"));
    let mut u = good.clone();
    u.bar.timestamp = -1;
    trials.push((u, ErrorCode::InvalidTimestamp, "timestamp[2]"));
    let mut u = good.clone();
    u.bar.available_at = 1000;
    trials.push((u, ErrorCode::InvalidAvailability, "available_at[2]"));
    let mut u = good.clone();
    u.bar.close = None;
    trials.push((u, ErrorCode::MissingData, "close[2]"));
    let mut u = good.clone();
    u.bar.volume = Some(FiniteF64::new(-1.0)?);
    trials.push((u, ErrorCode::NegativeVolume, "volume[2]"));
    let mut u = good.clone();
    u.bar.high = Some(FiniteF64::new(-1.0)?);
    trials.push((u, ErrorCode::InvalidOhlc, "ohlc[2]"));
    let mut u = good.clone();
    u.bar.is_closed = false;
    trials.push((u, ErrorCode::InvalidAvailability, "state.new_bar"));
    let mut u = good.clone();
    u.as_of = 1999;
    trials.push((u, ErrorCode::InvalidAvailability, "as_of"));
    let mut u = good.clone();
    u.as_of = MAX_TIMESTAMP + 1;
    trials.push((u, ErrorCode::InvalidTimestamp, "as_of"));
    let mut u = good.clone();
    u.metadata.gaps.push(DataGap {
        after_timestamp: 2000,
        before_timestamp: 3000,
    });
    trials.push((u, ErrorCode::ResetRequired, "metadata.gaps"));
    for (u, code, path) in trials {
        error(state.update_closed_bar(&u), code, path);
        assert_eq!(state, original);
    }
    Ok(())
}

#[test]
fn equal_availability_and_as_of_allow_appends_without_inventing_gaps() -> TestResult {
    let mut r = request()?;
    r.data.available_at.fill(10000);
    r.as_of = 10000;
    let mut initial = r.clone();
    support::truncate(&mut initial.data, 0);
    let mut state = AnalysisState::new(&initial)?;
    for i in 0..6 {
        state.update_closed_bar(&update(&state, &r.data, i, 10000))?;
    }
    assert_eq!(state.result(), &analyze_batch(&r)?);
    // The application owns the calendar. A later timestamp alone is no gap.
    let mut u = update(&state, &r.data, 5, 30000);
    u.bar.timestamp = 30000;
    u.bar.available_at = 30000;
    state.update_closed_bar(&u)?;
    assert_eq!(state.history_len(), 7);
    Ok(())
}

#[test]
fn engine_errors_during_update_and_reset_are_atomic() -> TestResult {
    let mut r = conflict()?;
    r.modules = vec![Module::MarketStructure];
    let mut initial = r.clone();
    initial.as_of = 8200;
    let mut state = AnalysisState::new(&initial)?;
    let before = state.clone();
    error(
        state.update_closed_bar(&update(&state, &r.data, 8, 9200)),
        ErrorCode::InvalidResult,
        "structure.breaks[8].references",
    );
    assert_eq!(state, before);
    error(
        state.reset(&r),
        ErrorCode::InvalidResult,
        "structure.breaks[8].references",
    );
    assert_eq!(state, before);
    let mut r = request()?;
    r.as_of = 2000;
    let mut state = AnalysisState::new(&r)?;
    let before = state.clone();
    let mut u = update(&state, &r.data, 2, 3000);
    u.bar.high = Some(FiniteF64::new(1e308)?);
    u.bar.low = Some(FiniteF64::new(-1e308)?);
    assert_eq!(
        state.update_closed_bar(&u).err().ok_or("overflow")?.code(),
        ErrorCode::NumericOverflow
    );
    assert_eq!(state, before);
    Ok(())
}

#[test]
fn revised_history_requires_reset_and_reset_replays_new_identity() -> TestResult {
    let r = request()?;
    let mut state = AnalysisState::new(&r)?;
    let previous = state.result().clone();
    let mut revised = r.clone();
    revised.data.metadata.data_revision = "r2".into();
    revised.data.metadata.snapshot_kind = SnapshotKind::RevisedHistory;
    revised.data.metadata.revision_available_at = 10000;
    revised.data.close[0] = Some(FiniteF64::new(1.5)?);
    revised.as_of = 10000;
    revised.config.indicators.period = 2;
    revised.modules = vec![Module::Indicators];
    let mut u = update(&state, &r.data, 5, 10000);
    u.metadata = revised.data.metadata.clone();
    error(
        state.update_closed_bar(&u),
        ErrorCode::StateMismatch,
        "state.identity",
    );
    let before = state.clone();
    let mut unavailable = revised.clone();
    unavailable.as_of = 9999;
    error(
        state.reset(&unavailable),
        ErrorCode::RevisionNotAvailable,
        "metadata.revision_available_at",
    );
    assert_eq!(state, before);
    let actual = state.reset(&revised)?;
    assert_eq!(actual, analyze_batch(&revised)?);
    assert_eq!(previous, analyze_batch(&r)?);
    assert_eq!(state.identity().data_revision, "r2");
    assert_ne!(state.identity().config_hash, before.identity().config_hash);
    // Reset may legitimately rewind as_of when rebuilding from the old snapshot.
    assert_eq!(state.reset(&r)?, previous);
    Ok(())
}

#[test]
fn update_wire_rejects_unknown_missing_nonfinite_and_bad_enum() -> TestResult {
    let r = request()?;
    let state = AnalysisState::new(&r)?;
    let u = update(&state, &r.data, 5, 7000);
    let wire = serde_json::to_value(&u)?;
    assert_eq!(u, serde_json::from_value(wire.clone())?);
    for (path, value) in [
        ("extra", json!(1)),
        ("bar", json!({})),
        ("identity", json!({})),
    ] {
        let mut bad = wire.clone();
        bad[path] = value;
        assert!(serde_json::from_value::<ClosedBarUpdate>(bad).is_err());
    }
    let mut bad = wire.clone();
    bad["bar"]["extra"] = json!(1);
    assert!(serde_json::from_value::<ClosedBarUpdate>(bad).is_err());
    let mut bad = wire;
    bad["identity"]["modules"] = json!(["unknown"]);
    assert!(serde_json::from_value::<ClosedBarUpdate>(bad).is_err());
    assert!(serde_json::from_str::<ClosedBar>(r#"{"timestamp":1,"available_at":1,"open":1e999,"high":1,"low":0,"close":1,"volume":0,"is_closed":true}"#).is_err());
    Ok(())
}

#[test]
fn reused_fixture_bytes_still_match_manifest() -> TestResult {
    let manifest: Value = serde_json::from_str(include_str!("fixtures/manifest.json"))?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for fixture in manifest["fixtures"].as_array().ok_or("fixtures")? {
        for file in fixture["files"].as_array().ok_or("files")? {
            let path = file["path"].as_str().ok_or("path")?;
            support::verify_checksum(
                path,
                &fs::read(root.join(path))?,
                file["sha256"].as_str().ok_or("sha")?,
            )?;
        }
    }
    Ok(())
}
