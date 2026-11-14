//! P2.1 Rust transport tests, distinct from the unverified native ABI smoke.
use serde_json::{json, Value};
use std::error::Error;
use wyc_binding_contract::{analyze_json, Handle};
use wyc_lib::{contracts::*, AnalysisRequest};

type TestResult = Result<(), Box<dyn Error>>;

fn request(n: usize) -> Result<AnalysisRequest, Box<dyn Error>> {
    let mut data: MarketData = serde_json::from_str(include_str!("fixtures/p1-arithmetic.json"))?;
    data.timestamp.truncate(n);
    data.available_at.truncate(n);
    data.open.truncate(n);
    data.high.truncate(n);
    data.low.truncate(n);
    data.close.truncate(n);
    data.volume.truncate(n);
    data.is_closed.truncate(n);
    let golden: Value = serde_json::from_str(include_str!("golden/p1-arithmetic.json"))?;
    Ok(AnalysisRequest {
        schema_version: SCHEMA_VERSION.into(),
        data,
        as_of: n as i64 * 1000,
        modules: vec![Module::Indicators, Module::MarketStructure],
        config: serde_json::from_value(golden["config"].clone())?,
    })
}

fn response(input: &str) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_str(input)?)
}

#[test]
fn batch_owned_wire_matches_core_and_independent_golden() -> TestResult {
    let r = request(6)?;
    let input = serde_json::to_string(&r)?;
    let result = response(&analyze_json(&input))?;
    assert_eq!(result["ok"], true);
    assert_eq!(
        result["value"],
        serde_json::to_value(wyc_lib::analyze_batch(&r)?)?
    );
    let golden: Value = serde_json::from_str(include_str!("golden/p1-arithmetic.json"))?;
    for i in [2, 3, 5] {
        for (name, expected) in golden[format!("scalars_at_index_{i}")]
            .as_object()
            .ok_or("scalars")?
        {
            let actual = result["value"]["indicators"]["series"][name][i]["result"]["value"]
                .as_f64()
                .ok_or("value")?;
            let expected = expected.as_f64().ok_or("golden")?;
            assert!((actual - expected).abs() <= 1e-12 + 1e-12 * expected.abs());
        }
    }
    assert_eq!(input, serde_json::to_string(&r)?);
    Ok(())
}

#[test]
fn warmup_missing_zero_and_empty_keep_distinct_shape() -> TestResult {
    let mut r = request(6)?;
    r.data.volume.fill(Some(FiniteF64::new(0.0)?));
    let result = response(&analyze_json(&serde_json::to_string(&r)?))?;
    let s = &result["value"]["indicators"]["series"];
    assert_eq!(
        s["sma"][0]["result"],
        json!({"status":"warmup","required":3,"available":1})
    );
    assert_eq!(
        s["rvol"][2]["result"],
        json!({"status":"missing","reason":"zero_denominator"})
    );
    assert_eq!(
        s["macd_histogram"][3]["result"],
        json!({"status":"ready","value":0.0})
    );
    let empty = response(&analyze_json(&serde_json::to_string(&request(0)?)?))?;
    assert_eq!(
        empty["value"]["indicators"]["series"]
            .as_object()
            .ok_or("series")?
            .len(),
        25
    );
    assert!(empty["value"]["indicators"]["series"]["sma"]
        .as_array()
        .ok_or("sma")?
        .is_empty());
    r.modules = vec![Module::Indicators];
    assert!(response(&analyze_json(&serde_json::to_string(&r)?))?["value"]["structure"].is_null());
    Ok(())
}

#[test]
fn strict_integer_missing_unknown_and_timestamp_errors() -> TestResult {
    let r = serde_json::to_value(request(6)?)?;
    for (path, value) in [
        ("as_of", json!(6000.5)),
        ("as_of", json!(true)),
        ("extra", json!(1)),
    ] {
        let mut invalid = r.clone();
        invalid[path] = value;
        assert_eq!(
            response(&analyze_json(&invalid.to_string()))?["error"],
            json!({"kind":"type","code":"invalid_type","path":"request"})
        );
    }
    let mut invalid = r.clone();
    invalid["as_of"] = json!(9_007_199_254_740_992_i64);
    assert_eq!(
        response(&analyze_json(&invalid.to_string()))?["error"],
        json!({"kind":"range","code":"invalid_timestamp","path":"as_of"})
    );
    invalid = r.clone();
    invalid["data"]["close"][0] = Value::Null;
    assert_eq!(
        response(&analyze_json(&invalid.to_string()))?["error"],
        json!({"kind":"type","code":"missing_data","path":"close[0]"})
    );
    invalid = r;
    invalid["config"]["indicators"]["period"] = json!(0);
    assert_eq!(
        response(&analyze_json(&invalid.to_string()))?["error"],
        json!({"kind":"range","code":"invalid_config","path":"config.indicators.period"})
    );
    Ok(())
}

#[test]
fn state_append_reset_errors_are_atomic_and_cursor_is_not_a_handle() -> TestResult {
    let mut handle = Handle::default();
    let full = request(6)?;
    let initial = serde_json::to_string(&request(3)?)?;
    assert_eq!(response(&handle.reset_json(&initial))?["ok"], true);
    let saved = handle.result_json();
    assert_eq!(
        response(&handle.update_json("{"))?["error"]["code"],
        "invalid_json"
    );
    assert_eq!(
        response(&handle.reset_json("{}"))?["error"]["code"],
        "invalid_type"
    );
    assert_eq!(handle.result_json(), saved);
    for i in 3..6 {
        let identity: StateIdentity =
            serde_json::from_value(response(&handle.identity_json())?["value"].clone())?;
        let update = ClosedBarUpdate {
            identity,
            metadata: full.data.metadata.clone(),
            as_of: (i as i64 + 1) * 1000,
            bar: ClosedBar {
                timestamp: full.data.timestamp[i],
                available_at: full.data.available_at[i],
                open: full.data.open[i],
                high: full.data.high[i],
                low: full.data.low[i],
                close: full.data.close[i],
                volume: full.data.volume[i],
                is_closed: true,
            },
        };
        let mut invalid = update.clone();
        invalid.identity.symbol = "different".into();
        let old = handle.result_json();
        assert_eq!(
            response(&handle.update_json(&serde_json::to_string(&invalid)?))?["error"],
            json!({"kind":"state","code":"state_mismatch","path":"state.identity"})
        );
        assert_eq!(old, handle.result_json());
        let result = response(&handle.update_json(&serde_json::to_string(&update)?))?;
        assert_eq!(
            result["value"],
            serde_json::to_value(wyc_lib::analyze_batch(&request(i + 1)?)?)?
        );
    }
    assert_ne!(handle.result_json(), saved);
    let old = handle.result_json();
    let mut invalid = full.clone();
    invalid.config.indicators.period = 0;
    assert_eq!(
        response(&handle.reset_json(&serde_json::to_string(&invalid)?))?["error"]["code"],
        "invalid_config"
    );
    assert_eq!(old, handle.result_json());
    let mut revised = full;
    revised.data.metadata.data_revision = "ramp-r2".into();
    revised.data.metadata.revision_available_at = 6000;
    revised.data.metadata.snapshot_kind = SnapshotKind::RevisedHistory;
    assert_eq!(
        response(&handle.reset_json(&serde_json::to_string(&revised)?))?["value"],
        serde_json::to_value(wyc_lib::analyze_batch(&revised)?)?
    );
    Ok(())
}

#[test]
fn independent_handles_owned_reports_close_is_permanent() -> TestResult {
    let mut a = Handle::default();
    let mut b = Handle::default();
    let input = serde_json::to_string(&request(3)?)?;
    a.reset_json(&input);
    b.reset_json(&input);
    let old = b.result_json();
    let mut caller_output = response(&a.result_json())?;
    caller_output["value"]["metadata"]["symbol"] = json!("caller-mutation");
    assert_eq!(a.result_json(), old);
    a.close();
    a.close();
    for output in [
        a.result_json(),
        a.identity_json(),
        a.reset_json("bad"),
        a.update_json("bad"),
    ] {
        assert_eq!(
            response(&output)?["error"],
            json!({"kind":"state","code":"state_closed","path":"state"})
        );
    }
    assert_eq!(b.result_json(), old);
    Ok(())
}

#[test]
fn unicode_and_exact_safe_timestamp_boundary_roundtrip() -> TestResult {
    let mut r = request(1)?;
    r.data.symbol = "Mã:Việt Nam".into();
    r.as_of = MAX_TIMESTAMP;
    r.data.timestamp[0] = MAX_TIMESTAMP;
    r.data.available_at[0] = MAX_TIMESTAMP;
    let result = response(&analyze_json(&serde_json::to_string(&r)?))?;
    assert_eq!(result["value"]["metadata"]["symbol"], "Mã:Việt Nam");
    assert_eq!(result["value"]["metadata"]["as_of"], json!(MAX_TIMESTAMP));
    Ok(())
}

#[test]
fn single_module_transport_matches_core_and_reports_errors() -> TestResult {
    for n in [0, 1, 3, 6] {
        let r = request(n)?;
        let input = serde_json::to_string(&r)?;
        let indicators = response(&wyc_binding_contract::indicators_json(&input))?;
        let structure = response(&wyc_binding_contract::structure_json(&input))?;
        assert_eq!(indicators["ok"], true);
        assert_eq!(structure["ok"], true);
        assert_eq!(
            indicators["value"],
            serde_json::to_value(wyc_lib::calculate_indicators(&r.data, &r.config, r.as_of)?)?
        );
        assert_eq!(
            structure["value"],
            serde_json::to_value(wyc_lib::analyze_structure(&r.data, &r.config, r.as_of)?)?
        );
    }
    for call in [
        wyc_binding_contract::indicators_json,
        wyc_binding_contract::structure_json,
    ] {
        assert_eq!(response(&call("{}"))?["error"]["code"], "invalid_type");
        let mut r = request(3)?;
        r.config.indicators.period = 0;
        assert_eq!(
            response(&call(&serde_json::to_string(&r)?))?["error"]["code"],
            "invalid_config"
        );
    }
    Ok(())
}
