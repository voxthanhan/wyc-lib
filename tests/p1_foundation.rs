use serde::Deserialize;
use serde_json::{json, Value};
use std::error::Error;
use wyc_lib::{contracts::*, prepare_request};

type TestResult = Result<(), Box<dyn Error>>;

fn request() -> Result<AnalysisRequest, Box<dyn Error>> {
    Ok(AnalysisRequest {
        schema_version: SCHEMA_VERSION.into(),
        data: serde_json::from_str(include_str!("fixtures/temporal.json"))?,
        as_of: 3500,
        modules: vec![Module::MarketStructure, Module::Indicators],
        config: AnalysisConfig::default(),
    })
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityFixture {
    config: AnalysisConfig,
    events: Vec<EventVector>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EventVector {
    engine_version: String,
    algorithm_version: String,
    symbol: String,
    timeframe: String,
    data_revision: String,
    source_module: Module,
    rule_id: String,
    event_type: String,
    observed_at: i64,
    known_at: i64,
    ordinal: u64,
}

impl EventVector {
    fn identity<'a>(&'a self, config_hash: &'a str) -> EventIdentity<'a> {
        EventIdentity {
            engine_version: &self.engine_version,
            algorithm_version: &self.algorithm_version,
            symbol: &self.symbol,
            timeframe: &self.timeframe,
            data_revision: &self.data_revision,
            config_hash,
            source_module: self.source_module,
            rule_id: &self.rule_id,
            event_type: &self.event_type,
            observed_at: self.observed_at,
            known_at: self.known_at,
            ordinal: self.ordinal,
        }
    }
}

fn identity_fixture() -> Result<(IdentityFixture, Value), Box<dyn Error>> {
    Ok((
        serde_json::from_str(include_str!("fixtures/p1-identity.json"))?,
        serde_json::from_str(include_str!("golden/p1-identity.json"))?,
    ))
}

#[test]
fn preparation_normalizes_selection_and_preserves_input() -> TestResult {
    let mut r = request()?;
    let before = serde_json::to_value(&r)?;
    let prepared = prepare_request(&r)?;
    assert_eq!(prepared.data().len(), 3);
    assert_eq!(prepared.data().as_of(), 3500);
    assert_eq!(
        prepared.modules(),
        [Module::Indicators, Module::MarketStructure]
    );
    assert!(std::ptr::eq(prepared.config(), &r.config));
    assert_eq!(prepared.metadata().schema_version, SCHEMA_VERSION);
    assert_eq!(prepared.metadata().engine_version, wyc_lib::ENGINE_VERSION);
    assert_eq!(
        prepared.metadata().algorithm_version,
        wyc_lib::ALGORITHM_VERSION
    );
    assert_eq!(prepared.metadata().symbol, r.data.symbol);
    assert_eq!(prepared.metadata().timeframe, r.data.timeframe);
    assert_eq!(
        prepared.metadata().data_revision,
        r.data.metadata.data_revision
    );
    assert_eq!(prepared.metadata().config_hash, r.config.config_hash()?);
    assert_eq!(prepared.metadata().as_of, r.as_of);
    let metadata = prepared.metadata().clone();
    assert_eq!(serde_json::to_value(&r)?, before);
    r.modules.reverse();
    assert_eq!(prepare_request(&r)?.metadata(), &metadata);
    r.modules = vec![Module::Indicators];
    assert_eq!(prepare_request(&r)?.modules(), [Module::Indicators]);
    r.modules = vec![Module::MarketStructure];
    assert_eq!(prepare_request(&r)?.modules(), [Module::MarketStructure]);
    Ok(())
}

#[test]
fn preparation_enforces_errors_and_validates_unavailable_tail() -> TestResult {
    let base = request()?;
    let mut cases = Vec::new();
    let mut r = base.clone();
    r.schema_version = "unknown".into();
    cases.push((r, ErrorCode::UnsupportedVersion, "schema_version"));
    let mut r = base.clone();
    r.as_of = -1;
    cases.push((r, ErrorCode::InvalidTimestamp, "as_of"));
    let mut r = base.clone();
    r.as_of = MAX_TIMESTAMP + 1;
    cases.push((r, ErrorCode::InvalidTimestamp, "as_of"));
    let mut r = base.clone();
    r.data.metadata.revision_available_at = 3501;
    cases.push((
        r,
        ErrorCode::RevisionNotAvailable,
        "metadata.revision_available_at",
    ));
    let mut r = base.clone();
    r.modules.clear();
    cases.push((r, ErrorCode::InvalidConfig, "modules"));
    let mut r = base.clone();
    r.modules = vec![Module::Indicators, Module::Indicators];
    cases.push((r, ErrorCode::DuplicateModule, "modules"));
    let mut r = base.clone();
    r.modules = vec![Module::PriceZones];
    cases.push((r, ErrorCode::UnsupportedModule, "modules"));
    let mut r = base.clone();
    r.config.indicators.period = 0;
    cases.push((r, ErrorCode::InvalidConfig, "config.indicators.period"));
    let mut r = base.clone();
    r.data.close.pop();
    cases.push((r, ErrorCode::LengthMismatch, "close"));
    let mut r = base.clone();
    r.as_of = 1000;
    r.data.close[3] = None;
    cases.push((r, ErrorCode::MissingData, "close[3]"));
    for (r, code, path) in cases {
        let before = serde_json::to_value(&r)?;
        assert_eq!(
            prepare_request(&r).map(|_| ()),
            Err(ContractError::new(code, path))
        );
        assert_eq!(serde_json::to_value(&r)?, before);
    }
    Ok(())
}

#[test]
fn replay_prefix_and_extreme_future_values_do_not_change_selected_data() -> TestResult {
    let base = request()?;
    for (as_of, expected_len) in [
        (0, 0),
        (999, 0),
        (1000, 1),
        (2000, 1),
        (2499, 1),
        (2500, 2),
        (3000, 2),
        (3499, 2),
        (3500, 3),
        (9000, 3),
        (MAX_TIMESTAMP, 3),
    ] {
        let mut r = base.clone();
        r.as_of = as_of;
        let full = prepare_request(&r)?;
        assert_eq!(full.data().len(), expected_len, "as_of={as_of}");
        assert_eq!(full.data().is_empty(), expected_len == 0);
        assert_eq!(full.data().bar(expected_len), None);
        assert_eq!(full.data().bar(usize::MAX), None);
        let mut prefix = r.clone();
        truncate(&mut prefix.data, expected_len);
        let short = prepare_request(&prefix)?;
        assert_eq!(short.metadata(), full.metadata());
        for index in 0..expected_len {
            assert_eq!(short.data().bar(index), full.data().bar(index));
        }
        let mut future = r.clone();
        for i in expected_len..future.data.timestamp.len() {
            future.data.open[i] = Some(FiniteF64::new(-1e200)?);
            future.data.high[i] = Some(FiniteF64::new(1e300)?);
            future.data.low[i] = Some(FiniteF64::new(-1e300)?);
            future.data.close[i] = Some(FiniteF64::new(1e200)?);
            future.data.volume[i] = Some(FiniteF64::new(1e300)?);
        }
        let changed = prepare_request(&future)?;
        assert_eq!(changed.metadata(), full.metadata());
        for i in 0..expected_len {
            assert_eq!(changed.data().bar(i), full.data().bar(i));
        }
    }
    Ok(())
}

#[test]
fn revision_and_delayed_availability_determine_sample_known_at() -> TestResult {
    let mut r = request()?;
    let view = prepare_request(&r)?;
    let delayed = view.data().numeric_sample(
        1,
        NumericValue::Ready {
            value: FiniteF64::new(0.0)?,
        },
    )?;
    assert_eq!((delayed.observed_at, delayed.known_at), (2000, 2500));
    r.data.metadata.revision_available_at = 6000;
    r.data.metadata.data_revision = "revised-r2".into();
    r.data.metadata.snapshot_kind = SnapshotKind::RevisedHistory;
    r.as_of = 6000;
    let revised = prepare_request(&r)?;
    for i in 0..revised.data().len() {
        assert_eq!(revised.data().bar(i).map(|b| b.known_at), Some(6000));
    }
    Ok(())
}

#[test]
fn numeric_series_preserves_statuses_and_rejects_shape_time_and_mask_drift() -> TestResult {
    let r = request()?;
    let view = AvailableData::new(&r.data, r.as_of)?;
    let statuses = [
        NumericValue::Warmup {
            required: 2,
            available: 1,
        },
        NumericValue::Ready {
            value: FiniteF64::new(0.0)?,
        },
        NumericValue::Missing {
            reason: MissingReason::ZeroDenominator,
        },
    ];
    let samples: Vec<_> = statuses
        .into_iter()
        .enumerate()
        .map(|(i, s)| view.numeric_sample(i, s))
        .collect::<Result<_, _>>()?;
    view.validate_numeric_series(&samples)?;
    let wire = serde_json::to_value(&samples)?;
    assert_eq!(
        wire[0]["result"],
        json!({"status":"warmup","required":2,"available":1})
    );
    assert_eq!(wire[1]["result"], json!({"status":"ready","value":0.0}));
    assert_eq!(
        wire[2]["result"],
        json!({"status":"missing","reason":"zero_denominator"})
    );
    assert_eq!(serde_json::from_value::<Vec<NumericSample>>(wire)?, samples);
    assert!(view.validate_numeric_series(&samples[..2]).is_err());
    let mut extra = samples.clone();
    extra.push(samples[0].clone());
    assert!(view.validate_numeric_series(&extra).is_err());
    for (index, observed, known) in [
        (0, 999, 1000),
        (1, 2000, 2000),
        (2, 3000, 3501),
        (0, 1000, 4000),
    ] {
        let mut bad = samples.clone();
        bad[index].observed_at = observed;
        bad[index].known_at = known;
        assert!(view.validate_numeric_series(&bad).is_err());
    }
    for result in [
        NumericValue::Warmup {
            required: 0,
            available: 0,
        },
        NumericValue::Warmup {
            required: 1,
            available: 1,
        },
        NumericValue::Warmup {
            required: 3,
            available: 2,
        },
    ] {
        assert!(view.numeric_sample(0, result).is_err());
    }
    assert!(view.numeric_sample(3, samples[0].result.clone()).is_err());
    let mut bad = samples.clone();
    bad[0].result = NumericValue::Warmup {
        required: 3,
        available: 2,
    };
    assert!(view.validate_numeric_series(&bad).is_err());
    view.numeric_sample(
        0,
        NumericValue::Missing {
            reason: MissingReason::UnavailableDependency,
        },
    )?;
    let mut empty = r.data.clone();
    truncate(&mut empty, 0);
    let view = AvailableData::new(&empty, 0)?;
    view.validate_numeric_series(&[])?;
    assert!(view.numeric_sample(0, samples[0].result.clone()).is_err());
    Ok(())
}

#[test]
fn canonical_config_and_event_ids_match_independent_vectors() -> TestResult {
    let (fixture, golden) = identity_fixture()?;
    assert_eq!(
        fixture.config.canonical_config()?,
        golden["canonical_config"]
    );
    let hash = fixture.config.config_hash()?;
    assert_eq!(hash, golden["config_hash"]);
    for (i, vector) in fixture.events.iter().enumerate() {
        let event = vector.identity(&hash);
        let expected = &golden["event_ids"][i];
        assert_eq!(stable_event_id(&event)?, *expected);
        assert_eq!(stable_event_id(&event)?, *expected);
    }
    assert_ne!(golden["event_ids"][2], golden["event_ids"][3]);
    Ok(())
}

#[test]
fn config_hash_is_order_independent_sensitive_to_every_field_and_validated() -> TestResult {
    let config = AnalysisConfig::default();
    let expected = config.config_hash()?;
    let reordered: AnalysisConfig = serde_json::from_str(
        r#"{"structure":{"right":2,"left":2},"indicators":{"bollinger_multiplier":2.0,"macd_signal":9,"macd_slow":26,"macd_fast":12,"period":14},"config_version":"p1-v1"}"#,
    )?;
    assert_eq!(reordered.config_hash()?, expected);
    for i in 0..7 {
        let mut c = config.clone();
        match i {
            0 => c.indicators.period += 1,
            1 => c.indicators.macd_fast += 1,
            2 => c.indicators.macd_slow += 1,
            3 => c.indicators.macd_signal += 1,
            4 => {
                c.indicators.bollinger_multiplier =
                    FiniteF64::new(f64::from_bits(2.0_f64.to_bits() + 1))?
            }
            5 => c.structure.left += 1,
            _ => c.structure.right += 1,
        }
        assert_ne!(c.config_hash()?, expected, "field {i}");
    }
    let mut invalid = config.clone();
    invalid.config_version = "unknown".into();
    assert!(invalid.config_hash().is_err());
    invalid = config.clone();
    invalid.indicators.bollinger_multiplier = FiniteF64::new(-0.0)?;
    assert!(invalid.canonical_config().is_err());
    for (period, ok) in [(0, false), (1, true), (1_000_000, true), (1_000_001, false)] {
        let mut c = config.clone();
        c.indicators.period = period;
        assert_eq!(c.config_hash().is_ok(), ok);
    }
    Ok(())
}

#[test]
fn event_id_is_sensitive_to_every_identity_field_and_uses_wire_module_names() -> TestResult {
    let (fixture, _) = identity_fixture()?;
    let hash = fixture.config.config_hash()?;
    let base = fixture.events[0].identity(&hash);
    let expected = stable_event_id(&base)?;
    let mut config = fixture.config.clone();
    config.structure.right += 1;
    let other_hash = config.config_hash()?;
    for i in 0..12 {
        let mut changed = base;
        match i {
            0 => changed.engine_version = "0.2.0",
            1 => changed.algorithm_version = "p1-v2",
            2 => changed.symbol = "OTHER",
            3 => changed.timeframe = "other",
            4 => changed.data_revision = "r2",
            5 => changed.config_hash = &other_hash,
            6 => changed.source_module = Module::Indicators,
            7 => changed.rule_id = "other-rule",
            8 => changed.event_type = "pivot_low",
            9 => changed.observed_at += 1,
            10 => changed.known_at += 1,
            _ => changed.ordinal += 1,
        }
        assert_ne!(stable_event_id(&changed)?, expected, "field {i}");
    }
    for module in [
        Module::Indicators,
        Module::MarketStructure,
        Module::PriceZones,
        Module::VolumeEngine,
        Module::RelativeStrength,
        Module::WyckoffEngine,
        Module::EvidenceEngine,
        Module::FeatureEngine,
    ] {
        assert_eq!(serde_json::to_value(module)?, json!(module.as_str()));
    }
    Ok(())
}

#[test]
fn event_identity_rejects_invalid_fields_hash_and_time() -> TestResult {
    let (fixture, _) = identity_fixture()?;
    let hash = fixture.config.config_hash()?;
    let base = fixture.events[0].identity(&hash);
    for i in 0..8 {
        let mut event = base;
        match i {
            0 => event.engine_version = " ",
            1 => event.algorithm_version = "",
            2 => event.symbol = "\n",
            3 => event.timeframe = "",
            4 => event.data_revision = "",
            5 => event.config_hash = "",
            6 => event.rule_id = " ",
            _ => event.event_type = "",
        }
        assert_eq!(
            stable_event_id(&event).map_err(|e| e.code),
            Err(ErrorCode::InvalidMetadata)
        );
    }
    for invalid in [
        "illustrative",
        "sha256:config-v1:",
        "sha256:config-v1:86FA00F140396F6AE60D9301E4358A47D3A60160DE7D875423E65AAD88C5BB1C",
        "sha256:config-v2:86fa00f140396f6ae60d9301e4358a47d3a60160de7d875423e65aad88c5bb1c",
    ] {
        let event = EventIdentity {
            config_hash: invalid,
            ..base
        };
        assert_eq!(
            stable_event_id(&event),
            Err(ContractError::new(
                ErrorCode::InvalidMetadata,
                "event.config_hash"
            ))
        );
    }
    for (observed_at, known_at, code, path) in [
        (-1, 3500, ErrorCode::InvalidTimestamp, "event.observed_at"),
        (
            2000,
            MAX_TIMESTAMP + 1,
            ErrorCode::InvalidTimestamp,
            "event.known_at",
        ),
        (3501, 3500, ErrorCode::InvalidEvidence, "event.known_at"),
    ] {
        assert_eq!(
            stable_event_id(&EventIdentity {
                observed_at,
                known_at,
                ..base
            }),
            Err(ContractError::new(code, path))
        );
    }
    stable_event_id(&EventIdentity {
        observed_at: 0,
        known_at: 0,
        ..base
    })?;
    Ok(())
}

#[test]
fn prepared_event_identity_is_stable_when_as_of_or_future_tail_changes() -> TestResult {
    let mut r = request()?;
    r.as_of = 2500;
    let prepared = prepare_request(&r)?;
    let event = |metadata: &ResultMetadata| -> Result<String, ContractError> {
        stable_event_id(&EventIdentity {
            engine_version: &metadata.engine_version,
            algorithm_version: &metadata.algorithm_version,
            symbol: &metadata.symbol,
            timeframe: &metadata.timeframe,
            data_revision: &metadata.data_revision,
            config_hash: &metadata.config_hash,
            source_module: Module::MarketStructure,
            rule_id: "sample-rule",
            event_type: "sample-event",
            observed_at: 2000,
            known_at: 2500,
            ordinal: 0,
        })
    };
    let before = event(prepared.metadata())?;
    r.as_of = 9000;
    r.data.close[3] = Some(FiniteF64::new(3.0)?);
    assert_eq!(event(prepare_request(&r)?.metadata())?, before);
    Ok(())
}
