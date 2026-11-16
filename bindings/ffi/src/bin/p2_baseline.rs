//! P2.6 measurement harness; clocks never participate in analytical decisions.
use serde_json::{json, Value};
use std::{error::Error, fs, hint::black_box, time::Instant};
use wyc_lib::AnalysisRequest;

type BenchResult<T> = Result<T, Box<dyn Error>>;

fn measure<T>(
    n: usize,
    name: &str,
    mut call: impl FnMut() -> BenchResult<T>,
) -> BenchResult<Value> {
    for _ in 0..2 {
        black_box(call()?);
    }
    let mut samples = Vec::new();
    for _ in 0..5 {
        let started = Instant::now();
        for _ in 0..3 {
            black_box(call()?);
        }
        samples.push(started.elapsed().as_nanos() as f64 / 3.0);
    }
    Ok(json!({"bars": n, "operation": name, "ns_per_op": samples}))
}

fn main() -> BenchResult<()> {
    let path = std::env::args().nth(1).ok_or("input path required")?;
    let input = fs::read_to_string(path)?;
    let values: Vec<Value> = serde_json::from_str(&input)?;
    let mut rows = Vec::new();
    let mut outputs = Vec::new();
    for value in values {
        let request: AnalysisRequest = serde_json::from_value(value)?;
        let n = request.data.timestamp.len();
        let encoded = serde_json::to_string(&request)?;
        let result = wyc_lib::analyze_batch(&request)?;
        let indicators =
            wyc_lib::calculate_indicators(&request.data, &request.config, request.as_of)?;
        let structure = wyc_lib::analyze_structure(&request.data, &request.config, request.as_of)?;
        let output = json!(result);
        if output["indicators"] != json!(indicators) || output["structure"] != json!(structure) {
            return Err("single-module output differs from batch".into());
        }
        rows.push(measure(n, "direct_batch", || {
            Ok(wyc_lib::analyze_batch(&request)?)
        })?);
        rows.push(measure(n, "direct_two_calls", || {
            Ok((
                wyc_lib::calculate_indicators(&request.data, &request.config, request.as_of)?,
                wyc_lib::analyze_structure(&request.data, &request.config, request.as_of)?,
            ))
        })?);
        rows.push(measure(n, "input_clone", || Ok(request.clone()))?);
        rows.push(measure(n, "input_decode", || {
            Ok(serde_json::from_str::<AnalysisRequest>(&encoded)?)
        })?);
        rows.push(measure(n, "output_encode", || {
            Ok(serde_json::to_string(&result)?)
        })?);
        rows.push(measure(n, "json_transport", || {
            Ok(wyc_binding_contract::analyze_json(&encoded))
        })?);
        outputs.push(output);
    }
    println!(
        "{}",
        json!({"runtime":"rust", "rows":rows, "outputs":outputs,
        "warmups":2, "samples":5, "iterations":3, "profile":"release"})
    );
    Ok(())
}
