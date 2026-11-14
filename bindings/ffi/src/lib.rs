//! Copy-safe P2.1 transport. All analytical decisions remain in wyc_lib.
use serde::{de::DeserializeOwned, Serialize};
use wyc_lib::{
    contracts::ErrorCode, AnalysisError, AnalysisRequest, AnalysisState, ClosedBarUpdate,
};

pub const BINDING_VERSION: &str = "p2-ffi-v1";

#[derive(Debug, Serialize)]
pub struct BindingError {
    pub kind: &'static str,
    pub code: String,
    pub path: String,
}

impl BindingError {
    fn adapter(kind: &'static str, code: &str, path: &str) -> Self {
        Self {
            kind,
            code: code.into(),
            path: path.into(),
        }
    }
}

impl From<AnalysisError> for BindingError {
    fn from(error: AnalysisError) -> Self {
        let kind = match error.code() {
            ErrorCode::MissingData => "type",
            ErrorCode::StateMismatch | ErrorCode::ResetRequired => "state",
            ErrorCode::NumericOverflow | ErrorCode::InvalidResult => "engine",
            _ => "range",
        };
        // ErrorCode serializes as a snake_case JSON string, never a Debug name.
        let code = serde_json::to_value(error.code())
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| "binding_failure".into());
        Self {
            kind,
            code,
            path: error.path().into(),
        }
    }
}

fn decode<T: DeserializeOwned>(input: &str, path: &str) -> Result<T, BindingError> {
    // Establish syntax independently of typed field failures (including trailing JSON).
    serde_json::from_str::<serde_json::Value>(input)
        .map_err(|_| BindingError::adapter("type", "invalid_json", path))?;
    serde_json::from_str(input).map_err(|e| {
        let code = if e.is_data() {
            "invalid_type"
        } else {
            "invalid_json"
        };
        BindingError::adapter("type", code, path)
    })
}

#[derive(Serialize)]
#[serde(untagged)]
enum Response<T> {
    Ok { ok: bool, value: T },
    Err { ok: bool, error: BindingError },
}

fn encode<T: Serialize>(result: Result<T, BindingError>) -> String {
    let response = match result {
        Ok(value) => Response::Ok { ok: true, value },
        Err(error) => Response::Err { ok: false, error },
    };
    serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"ok":false,"error":{"kind":"engine","code":"binding_failure","path":"result"}}"#.into()
    })
}

pub fn analyze_json(input: &str) -> String {
    encode(
        decode::<AnalysisRequest>(input, "request")
            .and_then(|request| wyc_lib::analyze_batch(&request).map_err(Into::into)),
    )
}

/// Single-module APIs preserve the exact Core signatures and metadata.
pub fn indicators_json(input: &str) -> String {
    encode(decode::<AnalysisRequest>(input, "request").and_then(|r| {
        wyc_lib::calculate_indicators(&r.data, &r.config, r.as_of).map_err(Into::into)
    }))
}

pub fn structure_json(input: &str) -> String {
    encode(
        decode::<AnalysisRequest>(input, "request").and_then(|r| {
            wyc_lib::analyze_structure(&r.data, &r.config, r.as_of).map_err(Into::into)
        }),
    )
}

/// Opaque runtime-owned handle; no registry, raw pointers or serialized caches.
#[derive(Default)]
pub struct Handle {
    state: Option<AnalysisState>,
    closed: bool,
}

impl Handle {
    fn live(&self) -> Result<(), BindingError> {
        if self.closed {
            Err(BindingError::adapter("state", "state_closed", "state"))
        } else {
            Ok(())
        }
    }

    fn state(&self) -> Result<&AnalysisState, BindingError> {
        self.live()?;
        self.state
            .as_ref()
            .ok_or_else(|| BindingError::adapter("state", "state_uninitialized", "state"))
    }

    pub fn result_json(&self) -> String {
        encode(self.state().map(AnalysisState::result))
    }

    pub fn identity_json(&self) -> String {
        encode(self.state().map(AnalysisState::identity))
    }

    pub fn reset_json(&mut self, input: &str) -> String {
        let result = (|| {
            self.live()?;
            let request = decode::<AnalysisRequest>(input, "request")?;
            let state = AnalysisState::new(&request).map_err(BindingError::from)?;
            // Encode the owned finite DTO; no runtime references are retained.
            let result = encode(Ok(state.result()));
            self.state = Some(state);
            Ok(result)
        })();
        match result {
            Ok(json) => json,
            Err(e) => encode::<()>(Err(e)),
        }
    }

    pub fn update_json(&mut self, input: &str) -> String {
        let result = (|| {
            self.state()?;
            let update = decode::<ClosedBarUpdate>(input, "update")?;
            let state = self
                .state
                .as_mut()
                .ok_or_else(|| BindingError::adapter("state", "state_uninitialized", "state"))?;
            state.update_closed_bar(&update).map_err(Into::into)
        })();
        encode(result)
    }

    pub fn close(&mut self) {
        self.state = None;
        self.closed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn strict_decode_and_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::from_str::<Value>(&analyze_json("{}"))?["error"]["code"],
            "invalid_type"
        );
        for input in ["{", "NaN", "{} {}", "1e999"] {
            assert_eq!(
                serde_json::from_str::<Value>(&analyze_json(input))?["error"]["code"],
                "invalid_json"
            );
        }
        let mut h = Handle::default();
        assert_eq!(
            serde_json::from_str::<Value>(&h.result_json())?["error"]["code"],
            "state_uninitialized"
        );
        h.close();
        h.close();
        assert_eq!(
            serde_json::from_str::<Value>(&h.reset_json("bad"))?["error"],
            json!({"kind":"state","code":"state_closed","path":"state"})
        );
        Ok(())
    }

    #[test]
    fn core_error_mapping_is_machine_readable() {
        let error = BindingError::from(AnalysisError::Contract(wyc_lib::ContractError::new(
            ErrorCode::ResetRequired,
            "metadata.gaps",
        )));
        assert_eq!(
            (error.kind, error.code.as_str(), error.path.as_str()),
            ("state", "reset_required", "metadata.gaps")
        );
    }
}
