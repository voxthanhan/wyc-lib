//! Native smoke reference, deliberately bypassing the FFI transport under test.
use std::io::{self, Read};
use wyc_lib::AnalysisRequest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: AnalysisRequest = serde_json::from_str(&input)?;
    println!(
        "{}",
        serde_json::to_string(&wyc_lib::analyze_batch(&request)?)?
    );
    Ok(())
}
