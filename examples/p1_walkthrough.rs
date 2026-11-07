use std::error::Error;
use wyc_lib::{contracts::*, *};

fn main() -> Result<(), Box<dyn Error>> {
    // Application supplies a normalized, versioned snapshot; no network or credentials.
    let data: MarketData =
        serde_json::from_str(include_str!("../tests/fixtures/p1-arithmetic.json"))?;
    let mut request = AnalysisRequest {
        schema_version: SCHEMA_VERSION.into(),
        data,
        as_of: 0,
        modules: vec![Module::Indicators, Module::MarketStructure],
        config: AnalysisConfig {
            indicators: IndicatorConfig {
                period: 3,
                macd_fast: 2,
                macd_slow: 3,
                macd_signal: 2,
                ..IndicatorConfig::default()
            },
            structure: StructureConfig { left: 1, right: 1 },
            ..AnalysisConfig::default()
        },
    };
    let mut state = AnalysisState::new(&request)?;
    for i in 0..request.data.timestamp.len() {
        let d = &request.data;
        let update = ClosedBarUpdate {
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
        };
        state.update_closed_bar(&update)?;
        request.as_of = update.as_of;
        assert_eq!(state.result(), &analyze_batch(&request)?);
    }
    let historical = state.result().clone();
    request.data.metadata.data_revision = "ramp-r2".into();
    request.data.metadata.snapshot_kind = SnapshotKind::RevisedHistory;
    request.data.metadata.revision_available_at = 7000;
    request.as_of = 7000;
    request.data.close[0] = Some(FiniteF64::new(1.5)?);
    assert_eq!(state.reset(&request)?, analyze_batch(&request)?);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "historical":historical,"after_reset":state.result()
        }))?
    );
    Ok(())
}
