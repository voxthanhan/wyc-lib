//! Reproducible P1 reference workload, not an optimization or a timing assertion.
use serde_json::{json, Value};
use std::{error::Error, hint::black_box, time::Instant};
use wyc_lib::{contracts::*, *};

type BenchResult<T> = Result<T, Box<dyn Error>>;

fn request(n: usize) -> BenchResult<AnalysisRequest> {
    let mut data: MarketData =
        serde_json::from_str(include_str!("../tests/fixtures/p1-arithmetic.json"))?;
    data.symbol = "BASELINE".into();
    data.metadata.source = "synthetic-triangle-v1".into();
    data.metadata.data_revision = "baseline-r1".into();
    data.timestamp = (1..=n).map(|i| i as i64 * 1000).collect();
    data.available_at = data.timestamp.iter().map(|t| t + 200).collect();
    // Exact integer-valued triangle: real strict pivots, equal-high/low labels, zero volume.
    let close: Vec<_> = (0..n)
        .map(|i| {
            let phase = (i % 32) as f64;
            FiniteF64::new(100.0 + phase.min(32.0 - phase)).map(Some)
        })
        .collect::<Result<_, _>>()?;
    data.open = close.clone();
    data.high = close
        .iter()
        .map(|v| v.map(|v| FiniteF64::new(v.get() + 1.0)).transpose())
        .collect::<Result<_, _>>()?;
    data.low = close
        .iter()
        .map(|v| v.map(|v| FiniteF64::new(v.get() - 1.0)).transpose())
        .collect::<Result<_, _>>()?;
    data.close = close;
    data.volume = (0..n)
        .map(|i| FiniteF64::new((i % 17 * 100) as f64).map(Some))
        .collect::<Result<_, _>>()?;
    data.is_closed = vec![true; n];
    Ok(AnalysisRequest {
        schema_version: SCHEMA_VERSION.into(),
        data,
        as_of: n as i64 * 1000 + 200,
        modules: vec![Module::Indicators, Module::MarketStructure],
        config: AnalysisConfig::default(),
    })
}

fn truncate(r: &mut AnalysisRequest, n: usize) {
    let d = &mut r.data;
    d.timestamp.truncate(n);
    d.available_at.truncate(n);
    d.open.truncate(n);
    d.high.truncate(n);
    d.low.truncate(n);
    d.close.truncate(n);
    d.volume.truncate(n);
    d.is_closed.truncate(n);
    r.as_of = d.available_at.last().copied().unwrap_or(0);
}

fn update(state: &AnalysisState, r: &AnalysisRequest, i: usize) -> ClosedBarUpdate {
    let d = &r.data;
    ClosedBarUpdate {
        identity: state.identity().clone(),
        metadata: d.metadata.clone(),
        as_of: d.available_at[i],
        bar: ClosedBar {
            timestamp: d.timestamp[i],
            available_at: d.available_at[i],
            open: d.open[i],
            high: d.high[i],
            low: d.low[i],
            close: d.close[i],
            volume: d.volume[i],
            is_closed: d.is_closed[i],
        },
    }
}

fn measure(
    name: &str,
    n: usize,
    units: usize,
    mut f: impl FnMut() -> BenchResult<f64>,
) -> BenchResult<Value> {
    black_box(f()?); // One unrecorded warm-up per workload.
    let mut samples = Vec::new();
    let mut checksum = 0.0;
    for _ in 0..5 {
        let start = Instant::now();
        checksum += black_box(f()?);
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let mut sorted = samples.clone();
    sorted.sort_by(f64::total_cmp);
    Ok(
        json!({"workload":name,"bars_per_symbol":n,"units_per_sample":units,
        "samples_ms":samples,"min_ms":sorted[0],"median_ms":sorted[2],"max_ms":sorted[4],
        "units_per_second":units as f64/(sorted[2]/1000.0),"checksum":checksum}),
    )
}

fn consume(result: AnalysisResult) -> f64 {
    let value = result
        .indicators
        .as_ref()
        .and_then(|r| r.series[&IndicatorKind::Sma].last())
        .map(|s| match s.result {
            NumericValue::Ready { value } => value.get(),
            _ => 0.0,
        })
        .unwrap_or(0.0);
    black_box(
        value
            + result
                .structure
                .as_ref()
                .map(|s| s.pivots.len())
                .unwrap_or(0) as f64,
    )
    // Owned output is dropped within the timed region, same as an application consuming it.
}

fn main() -> BenchResult<()> {
    let mut rows = Vec::new();
    for n in [256, 1024] {
        let r = request(n)?;
        for (name, modules) in [
            ("batch_indicators", vec![Module::Indicators]),
            ("batch_structure", vec![Module::MarketStructure]),
            ("batch_both", r.modules.clone()),
        ] {
            let mut selected = r.clone();
            selected.modules = modules;
            rows.push(measure(name, n, n, || {
                Ok(consume(analyze_batch(black_box(&selected))?))
            })?);
        }
        let mut prefix = r.clone();
        truncate(&mut prefix, n - 1);
        let initial = AnalysisState::new(&prefix)?;
        let u = update(&initial, &r, n - 1);
        // Clone/setup excluded; full returned output + state replacement/drop included.
        let mut append_samples = Vec::new();
        for _ in 0..6 {
            let mut state = initial.clone();
            let start = Instant::now();
            let result = state.update_closed_bar(black_box(&u))?;
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            black_box(consume(result));
            append_samples.push(elapsed);
        }
        append_samples.remove(0);
        let mut sorted = append_samples.clone();
        sorted.sort_by(f64::total_cmp);
        rows.push(
            json!({"workload":"append_one_both","bars_per_symbol":n,"units_per_sample":1,
            "samples_ms":append_samples,"min_ms":sorted[0],"median_ms":sorted[2],"max_ms":sorted[4],
            "units_per_second":1000.0/sorted[2]}),
        );
        let mut empty = r.clone();
        truncate(&mut empty, 0);
        rows.push(measure("replay_both", n, n, || {
            let mut state = AnalysisState::new(&empty)?;
            for i in 0..n {
                black_box(state.update_closed_bar(&update(&state, &r, i))?);
            }
            Ok(consume(state.result().clone()))
        })?);
        let watchlist: Vec<_> = (0..8)
            .map(|i| {
                let mut stock = r.clone();
                stock.data.symbol = format!("WATCH-{i}");
                stock
            })
            .collect();
        rows.push(measure("watchlist_8_sequential", n, n * 8, || {
            let mut sum = 0.0;
            for stock in &watchlist {
                sum += consume(analyze_batch(stock)?);
            }
            Ok(sum)
        })?);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "baseline_version":"p1-baseline-v1","algorithm_version":ALGORITHM_VERSION,
            "profile":"release","dataset":"synthetic-triangle-v1","threads":1,
            "config":AnalysisConfig::default(),"config_hash":AnalysisConfig::default().config_hash()?,
            "warmups":1,"samples_per_workload":5,"timing":"std::time::Instant; milliseconds",
            "rows":rows
        }))?
    );
    Ok(())
}
