//! P2.4 reference: direct public Core calls, never the binding transport.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{self, Read},
};
use wyc_lib::{AnalysisError, AnalysisRequest, AnalysisState, ClosedBarUpdate};

#[derive(Deserialize)]
struct Action {
    name: String,
    op: String,
    #[serde(default)]
    input: Value,
    #[serde(default)]
    identity_patch: BTreeMap<String, Value>,
}

fn execute(
    action: &Action,
    state: &mut Option<AnalysisState>,
) -> Result<Value, Box<dyn std::error::Error>> {
    // A malformed test protocol is a harness failure, not an analytical error.
    let result: Result<Value, AnalysisError> = match action.op.as_str() {
        "batch" | "indicators" | "structure" | "new" | "reset" => {
            let request: AnalysisRequest = serde_json::from_value(action.input.clone())?;
            match action.op.as_str() {
                "batch" => wyc_lib::analyze_batch(&request).map(|v| json!(v)),
                "indicators" => {
                    wyc_lib::calculate_indicators(&request.data, &request.config, request.as_of)
                        .map(|v| json!(v))
                }
                "structure" => {
                    wyc_lib::analyze_structure(&request.data, &request.config, request.as_of)
                        .map(|v| json!(v))
                }
                "new" => AnalysisState::new(&request).map(|s| {
                    let value = json!(s.result());
                    *state = Some(s);
                    value
                }),
                _ => state
                    .as_mut()
                    .ok_or("reset without state")?
                    .reset(&request)
                    .map(|v| json!(v)),
            }
        }
        "update" => {
            let current = state.as_mut().ok_or("update without state")?;
            let mut payload = action.input.clone();
            payload["identity"] = json!(current.identity());
            for (key, value) in &action.identity_patch {
                payload["identity"][key] = value.clone();
            }
            let update: ClosedBarUpdate = serde_json::from_value(payload)?;
            current.update_closed_bar(&update).map(|v| json!(v))
        }
        "result" => Ok(json!(state
            .as_ref()
            .ok_or("result without state")?
            .result())),
        "identity" => Ok(json!(state
            .as_ref()
            .ok_or("identity without state")?
            .identity())),
        _ => return Err("unknown parity operation".into()),
    };
    Ok(match result {
        Ok(value) => json!({"name":action.name,"value":value}),
        Err(error) => json!({"name":action.name,"error":{"code":error.code(),"path":error.path()}}),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let actions: Vec<Action> = serde_json::from_str(&input)?;
    let mut state = None;
    let records = actions
        .iter()
        .map(|action| execute(action, &mut state))
        .collect::<Result<Vec<_>, _>>()?;
    println!("{}", serde_json::to_string(&records)?);
    Ok(())
}
