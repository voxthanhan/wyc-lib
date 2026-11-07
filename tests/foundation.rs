mod support;

use serde::Deserialize;
use serde_json::{json, Value};
use std::{error::Error, fs, path::PathBuf};
use wyc_lib::contracts::*;

type TestResult = Result<(), Box<dyn Error>>;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn market() -> Result<MarketData, Box<dyn Error>> {
    Ok(serde_json::from_str(include_str!(
        "fixtures/temporal.json"
    ))?)
}
fn number(value: f64) -> Result<Option<FiniteF64>, ContractError> {
    Ok(Some(FiniteF64::new(value)?))
}
fn request() -> Result<AnalysisRequest, Box<dyn Error>> {
    Ok(AnalysisRequest {
        schema_version: SCHEMA_VERSION.into(),
        data: market()?,
        as_of: 3500,
        modules: vec![Module::Indicators, Module::MarketStructure],
        config: AnalysisConfig::default(),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    manifest_version: String,
    fixtures: Vec<Fixture>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    fixture_id: String,
    purpose: String,
    provenance: String,
    seed: Option<u64>,
    timezone: String,
    time_semantics: String,
    data_revision: String,
    config: Value,
    verification: String,
    files: Vec<FixtureFile>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureFile {
    path: String,
    sha256: String,
}

#[test]
fn manifest_checksums_and_fixture_schema() -> TestResult {
    let manifest: Manifest = serde_json::from_str(include_str!("fixtures/manifest.json"))?;
    assert_eq!(manifest.manifest_version, "1");
    assert_eq!(manifest.fixtures.len(), 6);
    for fixture in manifest.fixtures {
        assert!(!fixture.fixture_id.is_empty() && !fixture.purpose.is_empty());
        assert!(!fixture.provenance.is_empty() && !fixture.verification.is_empty());
        assert_eq!(fixture.seed, None);
        assert_eq!(fixture.timezone, "Etc/UTC");
        assert_eq!(fixture.time_semantics, "UTC Unix milliseconds at bar close");
        assert!(!fixture.data_revision.is_empty() && fixture.config.is_object());
        for file in fixture.files {
            let bytes = fs::read(root().join(&file.path))?;
            support::verify_checksum(&file.path, &bytes, &file.sha256)?;
            if file.path.starts_with("tests/fixtures/") {
                if fixture.fixture_id == "p1-advanced-v1" {
                    let advanced: Value = serde_json::from_slice(&bytes)?;
                    assert_eq!(advanced["fixture_id"], fixture.fixture_id);
                    assert_eq!(advanced["algorithm_version"], "p1-v1");
                    assert_eq!(
                        advanced["cases"].as_array().ok_or("advanced cases")?.len(),
                        8
                    );
                    continue;
                }
                if fixture.fixture_id == "p1-rolling-v1" || fixture.fixture_id == "p1-basic-v1" {
                    let rolling: Value = serde_json::from_slice(&bytes)?;
                    assert_eq!(rolling["fixture_id"], fixture.fixture_id);
                    assert_eq!(rolling["config"], fixture.config);
                    assert_eq!(rolling["algorithm_version"], "p1-v1");
                    continue;
                }
                if fixture.fixture_id == "p1-identity-v1" {
                    let identity: Value = serde_json::from_slice(&bytes)?;
                    let config: AnalysisConfig =
                        serde_json::from_value(identity["config"].clone())?;
                    config.validate()?;
                    assert_eq!(identity["config"], fixture.config);
                    continue;
                }
                let data: MarketData = serde_json::from_slice(&bytes)?;
                data.validate()?;
                assert_eq!(data.metadata.data_revision, fixture.data_revision);
            }
        }
    }
    Ok(())
}

#[test]
fn availability_replay_and_prefix_contract() -> TestResult {
    let data = market()?;
    let original = serde_json::to_value(&data)?;
    let golden: Value = serde_json::from_str(include_str!("golden/temporal.json"))?;
    let times: Vec<i64> = serde_json::from_value(golden["as_of"].clone())?;
    let expected: Vec<usize> = serde_json::from_value(golden["available_lengths"].clone())?;
    assert_eq!(times.len(), expected.len());
    for (as_of, len) in times.into_iter().zip(expected) {
        assert_eq!(data.available_len(as_of)?, len, "as_of={as_of}");
        let mut prefix = data.clone();
        support::truncate(&mut prefix, len);
        assert_eq!(prefix.available_len(as_of)?, len);
    }
    assert_eq!(serde_json::to_value(&data)?, original, "input mutated");
    let mut revised = data.clone();
    revised.metadata.data_revision = "synthetic-r2".into();
    revised.metadata.revision_available_at = 6000;
    revised.metadata.snapshot_kind = SnapshotKind::RevisedHistory;
    revised.close[0] = number(2.0)?;
    assert_eq!(
        revised.available_len(5500).map_err(|e| e.code),
        Err(ErrorCode::RevisionNotAvailable)
    );
    assert_eq!(revised.available_len(6000)?, 3);
    // Keep old revision available to replay old queries; revised data is a new input.
    assert_eq!(data.available_len(3500)?, 3);
    Ok(())
}

#[test]
fn invalid_data_counterexamples_have_exact_codes_and_paths() -> TestResult {
    let cases: Vec<(MarketData, ErrorCode, &str)> = {
        let base = market()?;
        let mut cases = Vec::new();
        let mut d = base.clone();
        d.schema_version = "9".into();
        cases.push((d, ErrorCode::UnsupportedVersion, "schema_version"));
        let mut d = base.clone();
        d.symbol = " ".into();
        cases.push((d, ErrorCode::InvalidMetadata, "symbol"));
        let mut d = base.clone();
        d.close.pop();
        cases.push((d, ErrorCode::LengthMismatch, "close"));
        let mut d = base.clone();
        d.timestamp[0] = -1;
        cases.push((d, ErrorCode::InvalidTimestamp, "timestamp[0]"));
        let mut d = base.clone();
        d.timestamp[1] = 1000;
        cases.push((d, ErrorCode::DuplicateTimestamp, "timestamp[1]"));
        let mut d = base.clone();
        d.timestamp[1] = 500;
        cases.push((d, ErrorCode::UnorderedTimestamp, "timestamp[1]"));
        let mut d = base.clone();
        d.available_at[1] = 1999;
        cases.push((d, ErrorCode::InvalidAvailability, "available_at[1]"));
        let mut d = base.clone();
        d.available_at[1] = 3600;
        cases.push((d, ErrorCode::InvalidAvailability, "available_at[2]"));
        let mut d = base.clone();
        d.is_closed[0] = false;
        cases.push((d, ErrorCode::InvalidAvailability, "available_at[1]"));
        let mut d = base.clone();
        d.close[0] = None;
        cases.push((d, ErrorCode::MissingData, "close[0]"));
        let mut d = base.clone();
        d.high[0] = number(0.0)?;
        cases.push((d, ErrorCode::InvalidOhlc, "ohlc[0]"));
        let mut d = base.clone();
        d.volume[0] = number(-1.0)?;
        cases.push((d, ErrorCode::NegativeVolume, "volume[0]"));
        let mut d = base.clone();
        d.metadata.gaps.push(DataGap {
            after_timestamp: 1000,
            before_timestamp: 2000,
        });
        cases.push((d, ErrorCode::DeclaredGap, "metadata.gaps"));
        cases
    };
    for (data, code, path) in cases {
        let before = serde_json::to_value(&data)?;
        assert_eq!(data.validate(), Err(ContractError::new(code, path)));
        assert_eq!(serde_json::to_value(&data)?, before);
    }
    assert_eq!(validate_timestamp(MAX_TIMESTAMP, "t"), Ok(()));
    assert!(validate_timestamp(MAX_TIMESTAMP + 1, "t").is_err());
    let mut empty = market()?;
    support::truncate(&mut empty, 0);
    assert_eq!(empty.available_len(0)?, 0);
    Ok(())
}

#[test]
fn all_columns_reject_length_mismatch_and_all_prices_reject_null() -> TestResult {
    let value = serde_json::to_value(market()?)?;
    for field in [
        "available_at",
        "open",
        "high",
        "low",
        "close",
        "volume",
        "is_closed",
    ] {
        let mut v = value.clone();
        v[field] = json!([]);
        let data: MarketData = serde_json::from_value(v)?;
        assert_eq!(
            data.validate(),
            Err(ContractError::new(ErrorCode::LengthMismatch, field))
        );
    }
    for field in ["open", "high", "low", "close", "volume"] {
        let mut v = value.clone();
        v[field][0] = Value::Null;
        let data: MarketData = serde_json::from_value(v)?;
        assert_eq!(
            data.validate(),
            Err(ContractError::new(
                ErrorCode::MissingData,
                format!("{field}[0]")
            ))
        );
    }
    Ok(())
}

#[test]
fn request_config_and_selection_contract() -> TestResult {
    let mut r = request()?;
    r.validate()?;
    let wire = serde_json::to_value(&r)?;
    let decoded: AnalysisRequest = serde_json::from_value(wire.clone())?;
    assert_eq!(r, decoded);
    r.modules.push(Module::Indicators);
    assert_eq!(
        r.validate().map_err(|e| e.code),
        Err(ErrorCode::DuplicateModule)
    );
    r.modules = vec![Module::WyckoffEngine];
    assert_eq!(
        r.validate().map_err(|e| e.code),
        Err(ErrorCode::UnsupportedModule)
    );
    r.modules.clear();
    assert_eq!(
        r.validate().map_err(|e| e.code),
        Err(ErrorCode::InvalidConfig)
    );
    r.modules = vec![Module::Indicators];
    r.config.indicators.period = 0;
    assert_eq!(
        r.validate().map_err(|e| e.code),
        Err(ErrorCode::InvalidConfig)
    );
    r.config = AnalysisConfig::default();
    r.config.indicators.macd_fast = r.config.indicators.macd_slow;
    assert!(r.validate().is_err());
    r.config = AnalysisConfig::default();
    r.config.structure.right = 0;
    assert!(r.validate().is_err());
    r.config = AnalysisConfig::default();
    r.config.indicators.bollinger_multiplier = FiniteF64::new(0.0)?;
    assert_eq!(
        r.validate().map_err(|e| e.code),
        Err(ErrorCode::InvalidConfig)
    );
    r.config = AnalysisConfig::default();
    r.config.indicators.period = 1_000_001;
    assert_eq!(
        r.validate(),
        Err(ContractError::new(
            ErrorCode::InvalidConfig,
            "config.indicators.period"
        ))
    );
    r.config = AnalysisConfig::default();
    r.config.config_version = "unknown".into();
    assert_eq!(
        r.validate().map_err(|e| e.code),
        Err(ErrorCode::UnsupportedVersion)
    );
    r.config = AnalysisConfig::default();
    r.as_of = -1;
    assert_eq!(
        r.validate(),
        Err(ContractError::new(ErrorCode::InvalidTimestamp, "as_of"))
    );
    let mut unknown = wire.clone();
    unknown["surprise"] = json!(true);
    assert!(serde_json::from_value::<AnalysisRequest>(unknown).is_err());
    let mut unknown = wire.clone();
    unknown["config"]["indicators"]["surprise"] = json!(1);
    assert!(serde_json::from_value::<AnalysisRequest>(unknown).is_err());
    let mut unknown = wire;
    unknown["modules"] = json!(["unknown"]);
    assert!(serde_json::from_value::<AnalysisRequest>(unknown).is_err());
    Ok(())
}

#[test]
fn finite_numbers_statuses_and_draft_report_roundtrip() -> TestResult {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            FiniteF64::new(value).map(|v| v.get()),
            Err(ContractError::new(ErrorCode::NonFinite, "value"))
        );
    }
    assert_eq!(FiniteF64::new(-0.0)?.get().to_bits(), 0.0_f64.to_bits());
    assert!(serde_json::from_str::<FiniteF64>("1e400").is_err());
    assert!(serde_json::from_str::<FiniteF64>("null").is_err());
    let golden: Value = serde_json::from_str(include_str!("golden/temporal.json"))?;
    let statuses: Vec<NumericValue> =
        serde_json::from_value(golden["numeric_status_examples"].clone())?;
    assert!(matches!(statuses[0], NumericValue::Ready { .. }));
    assert!(matches!(statuses[1], NumericValue::Warmup { .. }));
    assert!(matches!(statuses[2], NumericValue::Missing { .. }));
    support::compare(
        &golden["numeric_status_examples"],
        &serde_json::to_value(statuses)?,
        0.0,
        0.0,
    )?;
    assert!(serde_json::from_value::<NumericValue>(
        json!({"status":"ready","value":1,"reason":"x"})
    )
    .is_err());
    let evidence: Evidence = serde_json::from_value(golden["delayed_pivot"].clone())?;
    evidence.validate_at(3500)?;
    assert_eq!(
        evidence.validate_at(3499).map_err(|e| e.code),
        Err(ErrorCode::InvalidEvidence)
    );
    let mut invalid = evidence.clone();
    invalid.known_at = 1000;
    assert!(invalid.validate_at(3500).is_err());
    let report = DecisionReport {
        schema_version: SCHEMA_VERSION.into(),
        report_id: "draft-example".into(),
        symbol: "SYNTH".into(),
        timeframe: "synthetic-close-1s".into(),
        as_of: 3500,
        engine_version: wyc_lib::ENGINE_VERSION.into(),
        algorithm_version: "structure-strict-close-v1".into(),
        data_revision: "synthetic-r1".into(),
        config_hash: "illustrative-only-not-a-runtime-hash".into(),
        hypothesis: "draft-no-aggregation".into(),
        status: ReportStatus::Watching,
        evidence: vec![evidence],
        counter_evidence: vec![],
        pending_conditions: vec!["P5 rules pending".into()],
        confirmation: None,
        invalidation: None,
    };
    let wire = serde_json::to_string(&report)?;
    assert_eq!(serde_json::from_str::<DecisionReport>(&wire)?, report);
    Ok(())
}

#[test]
fn harness_detects_intentional_numeric_schema_time_mask_and_checksum_drift() -> TestResult {
    let expected = json!({"as_of":3500,"sample":{"status":"ready","value":2.0}});
    support::compare(&expected, &expected, 1e-12, 1e-12)?;
    let close = json!({"as_of":3500,"sample":{"status":"ready","value":2.0000000000001}});
    support::compare(&expected, &close, 1e-12, 1e-12)?;
    for actual in [
        json!({"as_of":3501,"sample":{"status":"ready","value":2.0}}),
        json!({"as_of":3500,"sample":{"status":"ready","value":2.1}}),
        json!({"as_of":3500,"sample":{"status":"warmup","required":3,"available":2}}),
        json!({"as_of":3500,"sample":{"status":"ready","value":2.0},"extra":true}),
    ] {
        let difference = support::compare(&expected, &actual, 1e-12, 1e-12);
        assert!(difference.is_err(), "intentional drift was not detected");
        if let Err(message) = difference {
            assert!(message.starts_with("$"));
        }
    }
    assert!(support::compare(&json!(3500), &json!(3501), 100.0, 100.0).is_err());
    assert!(support::compare(
        &json!({"as_of":3500.0}),
        &json!({"as_of":3501.0}),
        100.0,
        100.0
    )
    .is_err());
    assert!(support::compare(&json!(1), &json!(1), f64::NAN, 0.0).is_err());
    let digest = support::checksum(b"fixture");
    support::verify_checksum("negative-fixture", b"fixture", &digest)?;
    let corrupted = support::verify_checksum("negative-fixture", b"fixturE", &digest);
    assert!(corrupted.is_err());
    if let Err(message) = corrupted {
        assert!(message.starts_with("negative-fixture:"));
    }
    Ok(())
}

#[test]
fn typed_result_schema_and_warmup_invariants() -> TestResult {
    use std::collections::BTreeMap;
    let sample = NumericSample {
        observed_at: 1000,
        known_at: 1000,
        result: NumericValue::Warmup {
            required: 3,
            available: 1,
        },
    };
    sample.validate_at(1000)?;
    assert!(sample.validate_at(999).is_err());
    for value in [
        NumericValue::Warmup {
            required: 0,
            available: 0,
        },
        NumericValue::Warmup {
            required: 3,
            available: 3,
        },
    ] {
        assert_eq!(
            value.validate().map_err(|e| e.code),
            Err(ErrorCode::InvalidResult)
        );
    }
    let result = AnalysisResult {
        metadata: ResultMetadata {
            schema_version: SCHEMA_VERSION.into(),
            engine_version: wyc_lib::ENGINE_VERSION.into(),
            algorithm_version: "p1-v1".into(),
            symbol: "SYNTH".into(),
            timeframe: "synthetic-close-1s".into(),
            data_revision: "synthetic-r1".into(),
            config_hash: "illustrative-only-not-a-runtime-hash".into(),
            as_of: 1000,
        },
        indicators: Some(IndicatorResult {
            series: BTreeMap::from([(IndicatorKind::Sma, vec![sample])]),
        }),
        structure: None,
    };
    let wire = serde_json::to_value(&result)?;
    assert!(wire["structure"].is_null());
    assert_eq!(
        wire["indicators"]["series"]["sma"][0]["result"]["status"],
        "warmup"
    );
    assert_eq!(
        serde_json::from_value::<AnalysisResult>(wire.clone())?,
        result
    );
    let mut invalid = wire;
    invalid["indicators"]["series"]["future_unversioned_indicator"] = json!([]);
    assert!(serde_json::from_value::<AnalysisResult>(invalid).is_err());
    Ok(())
}
