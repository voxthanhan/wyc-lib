mod support;

use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::BTreeSet, error::Error, fs, path::PathBuf};
use wyc_lib::{contracts::*, market_structure::calculate_structure};

type TestResult = Result<(), Box<dyn Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    fixture_id: String,
    algorithm_version: String,
    cases: Vec<Case>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    data: MarketData,
    config: AnalysisConfig,
    as_of: i64,
}
fn cases() -> Result<Vec<Case>, Box<dyn Error>> {
    let f: Fixture = serde_json::from_str(include_str!("fixtures/p1-structure.json"))?;
    assert_eq!(f.fixture_id, "p1-structure-v1");
    assert_eq!(f.algorithm_version, "p1-v1");
    Ok(f.cases)
}
fn run(c: &Case) -> Result<ModuleResult<StructureResult>, AnalysisError> {
    calculate_structure(&c.data, &c.config, c.as_of)
}
fn value(x: f64) -> Result<Option<FiniteF64>, ContractError> {
    Ok(Some(FiniteF64::new(x)?))
}

#[test]
fn golden_all_fields_and_read_only_input() -> TestResult {
    let golden: Value = serde_json::from_str(include_str!("golden/p1-structure.json"))?;
    let cases = cases()?;
    assert_eq!(cases.len(), 15);
    for (c, g) in cases
        .iter()
        .zip(golden["cases"].as_array().ok_or("golden cases")?)
    {
        let original = c.data.clone();
        let config = c.config.clone();
        let result = run(c)?;
        assert_eq!(g["id"], c.id);
        support::compare(&g["result"], &serde_json::to_value(&result)?, 1e-12, 1e-12)
            .map_err(|e| format!("{}: {e}", c.id))?;
        // All discrete fields and copied prices must be exact, including tiny
        // levels. Only calculated strength receives the floating-point budget.
        let expected: ModuleResult<StructureResult> = serde_json::from_value(g["result"].clone())?;
        let mut exact = result.clone();
        for (actual, golden) in exact.output.breaks.iter_mut().zip(&expected.output.breaks) {
            actual.strength = golden.strength.clone();
        }
        for (actual, golden) in exact
            .output
            .evidence
            .iter_mut()
            .zip(&expected.output.evidence)
        {
            if let Some(strength) = golden.measured_values.get("strength") {
                actual.measured_values.insert("strength".into(), *strength);
            }
        }
        assert_eq!(exact, expected, "{} exact fields", c.id);
        let decoded: ModuleResult<StructureResult> =
            serde_json::from_value(serde_json::to_value(&result)?)?;
        assert_eq!(decoded, result);
        assert_eq!(c.data, original);
        assert_eq!(c.config, config);
    }
    Ok(())
}

#[test]
fn manifest_checks_structure_bytes() -> TestResult {
    let manifest: Value = serde_json::from_str(include_str!("fixtures/manifest.json"))?;
    let entry = manifest["fixtures"]
        .as_array()
        .ok_or("fixtures")?
        .iter()
        .find(|x| x["fixture_id"] == "p1-structure-v1")
        .ok_or("structure entry")?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for file in entry["files"].as_array().ok_or("files")? {
        let path = file["path"].as_str().ok_or("path")?;
        support::verify_checksum(
            path,
            &fs::read(root.join(path))?,
            file["sha256"].as_str().ok_or("sha")?,
        )?;
    }
    Ok(())
}

#[test]
fn hand_traced_labels_trends_and_break_precedence() -> TestResult {
    let c = &cases()?[0];
    let out = run(c)?.output;
    assert_eq!(
        out.pivots
            .iter()
            .map(|p| (p.observed_at, p.label))
            .collect::<Vec<_>>(),
        vec![
            (2000, PivotLabel::Unclassified),
            (3000, PivotLabel::Unclassified),
            (4000, PivotLabel::Lh),
            (5000, PivotLabel::Hl),
            (6000, PivotLabel::Hh),
            (7000, PivotLabel::Hl),
            (10000, PivotLabel::Hh),
            (13000, PivotLabel::Ll),
            (14000, PivotLabel::Lh),
            (15000, PivotLabel::Ll),
            (16000, PivotLabel::Lh),
            (17000, PivotLabel::Ll)
        ]
    );
    assert_eq!(out.trends[5].trend, Trend::Sideways);
    assert_eq!(out.trends[6].trend, Trend::Uptrend);
    // Break at 13000 uses the previous uptrend; low 13000 confirms at 14000.
    assert_eq!(out.trends[12].trend, Trend::Uptrend);
    assert_eq!(out.trends[13].trend, Trend::Sideways);
    assert_eq!(out.trends[14].trend, Trend::Downtrend);
    assert_eq!(
        out.breaks
            .iter()
            .map(|b| (b.observed_at, b.kind, b.direction))
            .collect::<Vec<_>>(),
        vec![
            (9000, BreakKind::Bos, Direction::Up),
            (13000, BreakKind::Choch, Direction::Down),
            (15000, BreakKind::Bos, Direction::Down),
            (17000, BreakKind::Bos, Direction::Down),
            (19000, BreakKind::Choch, Direction::Up)
        ]
    );
    assert_eq!(
        out.breaks[0].strength,
        NumericValue::Ready {
            value: FiniteF64::new(0.25)?
        }
    );
    assert_eq!(out.breaks[0].known_at, 9200);
    // Close 16000 does not consume the already broken high a second time;
    // low wick at 11000 and equality at 12000 do not confirm a break.
    assert!(!out
        .breaks
        .iter()
        .any(|b| [10000, 11000, 12000].contains(&b.observed_at)));
    assert_eq!(out.trends[18].trend, Trend::Downtrend); // CHoCH does not overwrite trend.
    Ok(())
}

#[test]
fn equal_outside_pivots_have_both_records_and_one_trend_sample() -> TestResult {
    let c = &cases()?[2];
    let out = run(c)?.output;
    assert_eq!(out.pivots[0].kind, PivotKind::High);
    assert_eq!(out.pivots[1].kind, PivotKind::Low);
    assert_eq!(out.pivots[0].observed_at, out.pivots[1].observed_at);
    assert_eq!(out.pivots[2].label, PivotLabel::Eh);
    assert_eq!(out.pivots[3].label, PivotLabel::El);
    assert_eq!(out.trends.len(), c.data.timestamp.len());
    assert_eq!(out.trends[4].trend, Trend::Sideways);
    assert_ne!(out.pivots[0].pivot_id, out.pivots[1].pivot_id);
    Ok(())
}

#[test]
fn every_prefix_replays_with_future_changed_and_append_keeps_history() -> TestResult {
    for c in cases()? {
        let final_result = run(&c)?;
        for n in 0..=c.data.timestamp.len() {
            let as_of = if n == 0 {
                0
            } else {
                c.data.available_at[n - 1]
            };
            let mut prefix = c.data.clone();
            support::truncate(&mut prefix, n);
            let expected = calculate_structure(&prefix, &c.config, as_of)?;
            assert_eq!(
                calculate_structure(&c.data, &c.config, as_of)?,
                expected,
                "{} prefix {n}",
                c.id
            );
            let mut future = c.data.clone();
            for i in n..future.timestamp.len() {
                future.open[i] = value(1e120)?;
                future.high[i] = value(1e120)?;
                future.low[i] = value(1e120)?;
                future.close[i] = value(1e120)?;
            }
            assert_eq!(calculate_structure(&future, &c.config, as_of)?, expected);
            let mut history = final_result.output.clone();
            history.pivots.retain(|p| p.known_at <= as_of);
            history.breaks.retain(|p| p.known_at <= as_of);
            history.evidence.retain(|p| p.known_at <= as_of);
            history.trends.retain(|p| p.known_at <= as_of);
            assert_eq!(history, expected.output, "{} history {n}", c.id);
        }
    }
    Ok(())
}

#[test]
fn right_confirmation_waits_for_closed_available_bar() -> TestResult {
    let mut c = cases()?.remove(4); // L=1 R=2: high at 2000 confirms on bar 4000.
    c.data.available_at[3] = 4500;
    for (as_of, count) in [(2000, 0), (4000, 0), (4499, 0), (4500, 1), (4999, 1)] {
        assert_eq!(
            calculate_structure(&c.data, &c.config, as_of)?
                .output
                .pivots
                .len(),
            count
        );
    }
    let pivot = &calculate_structure(&c.data, &c.config, 4500)?.output.pivots[0];
    assert_eq!((pivot.observed_at, pivot.known_at), (2000, 4500));
    support::truncate(&mut c.data, 4);
    c.data.is_closed[3] = false;
    assert!(calculate_structure(&c.data, &c.config, 4500)?
        .output
        .pivots
        .is_empty());
    Ok(())
}

#[test]
fn revisions_change_timing_and_identity_and_require_availability() -> TestResult {
    let mut c = cases()?.remove(0);
    let old = run(&c)?;
    c.data.metadata.data_revision = "structure-r2".into();
    c.data.metadata.revision_available_at = 10000;
    let err = calculate_structure(&c.data, &c.config, 9999)
        .err()
        .ok_or("revision error")?;
    assert_eq!(err.code(), ErrorCode::RevisionNotAvailable);
    let new = run(&c)?;
    assert!(new.output.pivots.iter().all(|p| p.known_at >= 10000));
    assert!(new.output.breaks.iter().all(|p| p.known_at >= 10000));
    assert!(new.output.trends.iter().all(|p| p.known_at >= 10000));
    assert_ne!(new.output.pivots[0].pivot_id, old.output.pivots[0].pivot_id);
    c.data.high[1] = value(13.0)?;
    assert_eq!(run(&c)?, run(&c)?); // Stateless recomputation on revised snapshot.
    assert_ne!(old.output.pivots[0].level, run(&c)?.output.pivots[0].level);
    Ok(())
}

#[test]
fn lineage_ids_references_and_evidence_order_are_stable() -> TestResult {
    for c in cases()? {
        let result = run(&c)?;
        assert_eq!(result.metadata.config_hash, c.config.config_hash()?);
        assert_eq!(result.metadata.engine_version, wyc_lib::ENGINE_VERSION);
        assert_eq!(
            result.metadata.algorithm_version,
            wyc_lib::ALGORITHM_VERSION
        );
        let out = &result.output;
        let mut ids = BTreeSet::new();
        for b in &out.breaks {
            assert!(ids.insert(&b.reference_pivot_id));
            let p = out
                .pivots
                .iter()
                .find(|p| p.pivot_id == b.reference_pivot_id)
                .ok_or("reference")?;
            assert!(p.known_at <= b.known_at && p.observed_at < b.observed_at);
            b.strength.validate()?;
        }
        assert_eq!(out.evidence.len(), out.pivots.len() + out.breaks.len());
        for e in &out.evidence {
            e.validate_at(c.as_of)?;
            assert_eq!(e.status, ReportStatus::Confirmed);
        }
        let keys: Vec<_> = out
            .evidence
            .iter()
            .map(|e| {
                (
                    e.observed_at,
                    e.known_at,
                    e.source_module.as_str(),
                    &e.rule_id,
                    &e.evidence_id,
                )
            })
            .collect();
        assert!(keys.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(run(&c)?, result);
        let later = calculate_structure(&c.data, &c.config, c.as_of + 500)?;
        assert_eq!(later.output, result.output);
    }
    Ok(())
}

#[test]
fn gaps_zero_denominator_and_strength_are_not_probabilities() -> TestResult {
    let cases = cases()?;
    let out = run(&cases[7])?.output;
    assert_eq!(
        out.breaks[0].strength,
        NumericValue::Missing {
            reason: MissingReason::ZeroDenominator
        }
    );
    let e = out
        .evidence
        .iter()
        .find(|e| e.evidence_id == out.breaks[0].event_id)
        .ok_or("evidence")?;
    assert!(!e.measured_values.contains_key("strength"));
    assert_eq!(
        run(&cases[8])?.output.breaks[0].strength,
        NumericValue::Ready {
            value: FiniteF64::new(7.0)?
        }
    );
    assert!(run(&cases[9])?.output.breaks.is_empty()); // Unknown trend never consumes a pivot.
    Ok(())
}

#[test]
fn config_and_invalid_data_keep_contract_errors_including_future() -> TestResult {
    let c = cases()?.remove(0);
    let mut errors = Vec::new();
    let mut d = c.data.clone();
    d.close[0] = None;
    errors.push((d, ErrorCode::MissingData));
    let mut d = c.data.clone();
    d.close.pop();
    errors.push((d, ErrorCode::LengthMismatch));
    let mut d = c.data.clone();
    d.timestamp[1] = d.timestamp[0];
    errors.push((d, ErrorCode::DuplicateTimestamp));
    let mut d = c.data.clone();
    d.volume[0] = value(-1.0)?;
    errors.push((d, ErrorCode::NegativeVolume));
    let mut d = c.data.clone();
    d.high[0] = value(-100.0)?;
    errors.push((d, ErrorCode::InvalidOhlc));
    let mut d = c.data.clone();
    d.available_at[0] = 0;
    errors.push((d, ErrorCode::InvalidAvailability));
    for (d, code) in errors {
        let original = d.clone();
        assert_eq!(
            calculate_structure(&d, &c.config, 0)
                .err()
                .ok_or("data error")?
                .code(),
            code
        );
        assert_eq!(d, original);
    }
    for (left, right) in [(0, 1), (1, 0), (1000001, 1), (1, 1000001)] {
        let mut config = c.config.clone();
        config.structure = StructureConfig { left, right };
        assert_eq!(
            calculate_structure(&c.data, &config, c.as_of)
                .err()
                .ok_or("config error")?
                .code(),
            ErrorCode::InvalidConfig
        );
    }
    let mut config = c.config.clone();
    config.indicators.period = 0;
    assert_eq!(
        calculate_structure(&c.data, &config, c.as_of)
            .err()
            .ok_or("full config error")?
            .code(),
        ErrorCode::InvalidConfig
    );
    assert!(FiniteF64::new(f64::NAN).is_err());
    let mut wire = serde_json::to_value(&c.data)?;
    wire["high"][0] = json!("NaN");
    assert!(serde_json::from_value::<MarketData>(wire).is_err());
    Ok(())
}

#[test]
fn empty_short_plateau_and_max_windows_do_not_invent_pivots() -> TestResult {
    let cases = cases()?;
    for i in [3, 10, 11, 12] {
        let result = run(&cases[i])?;
        assert!(result.output.pivots.is_empty());
        assert!(result.output.breaks.is_empty());
        assert!(result
            .output
            .trends
            .iter()
            .all(|s| s.trend == Trend::Unknown));
    }
    let mut c = cases[0].clone();
    c.config.structure = StructureConfig {
        left: 1000000,
        right: 1000000,
    };
    assert!(run(&c)?.output.pivots.is_empty());
    Ok(())
}

#[test]
fn arithmetic_overflow_is_reported_without_partial_result() -> TestResult {
    let c = cases()?.remove(0);
    let mut d = c.data.clone();
    support::truncate(&mut d, 9);
    // Overflowing range on the first break.
    d.high[8] = value(f64::MAX)?;
    d.low[8] = value(-f64::MAX)?;
    d.close[8] = value(20.0)?;
    let err = calculate_structure(&d, &c.config, 9200)
        .err()
        .ok_or("range overflow")?;
    assert_eq!(err.code(), ErrorCode::NumericOverflow);
    assert_eq!(err.path(), "structure.breaks[8].strength");
    assert_eq!(
        calculate_structure(&d, &c.config, 8199)?
            .output
            .breaks
            .len(),
        0
    );
    Ok(())
}

fn crossed_references() -> Result<Case, Box<dyn Error>> {
    let mut c = cases()?.remove(0);
    support::truncate(&mut c.data, 9);
    for (target, values) in [
        (
            &mut c.data.high,
            vec![6., 10., 7., 9., 8., 20., 21., 22., 23.],
        ),
        (
            &mut c.data.low,
            vec![4., 5., 4., 5., 6., 18., 17., 18., 11.],
        ),
        (
            &mut c.data.close,
            vec![5., 6., 5., 6., 7., 19., 19., 20., 12.],
        ),
    ] {
        *target = values.into_iter().map(value).collect::<Result<_, _>>()?;
    }
    c.data.open = c.data.close.clone();
    c.as_of = 9200;
    Ok(c)
}

#[test]
fn simultaneous_opposite_breaks_fail_instead_of_selecting_a_direction() -> TestResult {
    let c = crossed_references()?;
    let before = calculate_structure(&c.data, &c.config, 8200)?;
    assert!(before.output.breaks.is_empty());
    assert_eq!(before.output.trends[7].trend, Trend::Sideways);
    // Latest high=9 and low=17, both unconsumed; close=12 triggers both.
    let original = c.data.clone();
    let err = run(&c).err().ok_or("conflict")?;
    assert_eq!(err.code(), ErrorCode::InvalidResult);
    assert_eq!(err.path(), "structure.breaks[8].references");
    assert_eq!(c.data, original);
    Ok(())
}

#[test]
fn unknown_trend_does_not_consume_a_crossed_reference() -> TestResult {
    let mut c = crossed_references()?;
    c.data.low[8] = value(18.0)?;
    c.data.close[8] = value(20.0)?;
    c.data.open[8] = value(20.0)?;
    // Close 19/20 crossed high=9 on bars 6000..8000 while trend was
    // unknown; it is still available to break on the first known-trend bar.
    let out = run(&c)?.output;
    assert_eq!(out.breaks.len(), 1);
    assert_eq!(
        (out.breaks[0].observed_at, out.breaks[0].direction),
        (9000, Direction::Up)
    );
    assert_eq!(out.breaks[0].level, FiniteF64::new(9.0)?);
    Ok(())
}

#[test]
fn strength_subtraction_division_overflow_and_underflow_have_explicit_policy() -> TestResult {
    let c = cases()?.remove(0);
    let mut d = c.data.clone();
    support::truncate(&mut d, 9);
    // Division overflow with a finite distance and tiny positive range.
    d.high[8] = value(-1e-310)?;
    d.low[8] = value(-2e-310)?;
    d.open[8] = value(-1e-310)?;
    d.close[8] = value(-1e-310)?;
    let err = calculate_structure(&d, &c.config, 9200)
        .err()
        .ok_or("division overflow")?;
    assert_eq!(err.code(), ErrorCode::NumericOverflow);
    assert_eq!(err.path(), "structure.breaks[8].strength");
    // Subtraction overflow: shift all previous levels negative while
    // preserving pivots, then gap to a finite positive price.
    for column in [&mut d.high, &mut d.low, &mut d.close, &mut d.open] {
        for v in column.iter_mut().take(8) {
            *v = value(v.ok_or("price")?.get() * 1e306 - 1e308)?;
        }
    }
    d.high[8] = value(1.7e308)?;
    d.low[8] = value(1.6e308)?;
    d.close[8] = value(1.65e308)?;
    d.open[8] = value(1.65e308)?;
    assert_eq!(
        calculate_structure(&d, &c.config, 9200)
            .err()
            .ok_or("subtraction overflow")?
            .code(),
        ErrorCode::NumericOverflow
    );
    // Tiny finite distance / huge range may underflow to ready zero.
    let mut d = c.data.clone();
    support::truncate(&mut d, 9);
    for column in [&mut d.high, &mut d.low, &mut d.close, &mut d.open] {
        for v in column.iter_mut().take(8) {
            *v = value(v.ok_or("price")?.get() * 1e-310)?;
        }
    }
    d.high[8] = value(1e308)?;
    d.low[8] = value(0.0)?;
    d.close[8] = value(15e-310)?;
    d.open[8] = value(15e-310)?;
    assert_eq!(
        calculate_structure(&d, &c.config, 9200)?.output.breaks[0].strength,
        NumericValue::Ready {
            value: FiniteF64::new(0.0)?
        }
    );
    Ok(())
}

#[test]
fn declared_gap_is_rejected_and_config_changes_event_identity() -> TestResult {
    let mut c = cases()?.remove(0);
    let old = run(&c)?;
    c.config.indicators.period = 13; // Included in whole-config lineage, not executed.
    let new = run(&c)?;
    assert_eq!(new.output.pivots[0].level, old.output.pivots[0].level);
    assert_ne!(new.output.pivots[0].pivot_id, old.output.pivots[0].pivot_id);
    c.data.metadata.gaps.push(DataGap {
        after_timestamp: 1000,
        before_timestamp: 2000,
    });
    assert_eq!(
        run(&c).err().ok_or("declared gap")?.code(),
        ErrorCode::DeclaredGap
    );
    Ok(())
}
