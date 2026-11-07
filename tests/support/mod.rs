use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn verify_checksum(path: &str, bytes: &[u8], expected: &str) -> Result<(), String> {
    let actual = checksum(bytes);
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{path}: checksum expected {expected}, actual {actual}"
        ))
    }
}

/// Compare shape/status/strings exactly, finite numeric values with a declared budget.
/// Returns the first differing JSON path, suitable for a regression failure report.
pub fn compare(expected: &Value, actual: &Value, atol: f64, rtol: f64) -> Result<(), String> {
    fn walk(e: &Value, a: &Value, path: &str, atol: f64, rtol: f64) -> Result<(), String> {
        match (e, a) {
            (Value::Number(en), Value::Number(an)) => {
                // Timestamp/count fields remain exact even if represented as JSON floats.
                let exact_field = [
                    ".as_of",
                    ".observed_at",
                    ".known_at",
                    ".required",
                    ".available",
                ]
                .iter()
                .any(|suffix| path.ends_with(suffix))
                    || path.contains(".timestamp[")
                    || path.contains(".available_at[");
                let equal = if exact_field {
                    en.as_f64() == an.as_f64()
                } else if en.is_i64() && an.is_i64() {
                    en == an
                } else if let (Some(x), Some(y)) = (en.as_f64(), an.as_f64()) {
                    x.is_finite()
                        && y.is_finite()
                        && (x - y).abs() <= atol + rtol * x.abs().max(y.abs())
                } else {
                    false
                };
                if equal {
                    Ok(())
                } else {
                    Err(format!("{path}: expected {e}, actual {a}"))
                }
            }
            (Value::Array(es), Value::Array(as_)) if es.len() == as_.len() => {
                for (i, (ev, av)) in es.iter().zip(as_).enumerate() {
                    walk(ev, av, &format!("{path}[{i}]"), atol, rtol)?;
                }
                Ok(())
            }
            (Value::Object(es), Value::Object(as_)) if es.keys().eq(as_.keys()) => {
                for (key, ev) in es {
                    walk(ev, &as_[key], &format!("{path}.{key}"), atol, rtol)?;
                }
                Ok(())
            }
            _ if e == a => Ok(()),
            _ => Err(format!("{path}: expected {e}, actual {a}")),
        }
    }
    if !atol.is_finite() || !rtol.is_finite() || atol < 0.0 || rtol < 0.0 {
        return Err("$: invalid comparison tolerance".into());
    }
    walk(expected, actual, "$", atol, rtol)
}

pub fn truncate(data: &mut wyc_lib::MarketData, len: usize) {
    data.timestamp.truncate(len);
    data.available_at.truncate(len);
    data.open.truncate(len);
    data.high.truncate(len);
    data.low.truncate(len);
    data.close.truncate(len);
    data.volume.truncate(len);
    data.is_closed.truncate(len);
}
