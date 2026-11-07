//! Pure, versioned identity utilities shared by orchestration and algorithms.
use crate::{validate_timestamp, AnalysisConfig, ContractError, ErrorCode, Module};
use sha2::{Digest, Sha256};

const CONFIG_HASH_PREFIX: &str = "sha256:config-v1:";

impl AnalysisConfig {
    /// Canonical config-v1 bytes are ASCII, use LF, and include the final LF.
    pub fn canonical_config(&self) -> Result<String, ContractError> {
        self.validate()?;
        Ok(format!(
            "config_version={}\nperiod={}\nmacd_fast={}\nmacd_slow={}\nmacd_signal={}\nbollinger_multiplier_bits={:016x}\nleft={}\nright={}\n",
            self.config_version,
            self.indicators.period,
            self.indicators.macd_fast,
            self.indicators.macd_slow,
            self.indicators.macd_signal,
            self.indicators.bollinger_multiplier.get().to_bits(),
            self.structure.left,
            self.structure.right,
        ))
    }

    pub fn config_hash(&self) -> Result<String, ContractError> {
        let canonical = self.canonical_config()?;
        Ok(format!(
            "{CONFIG_HASH_PREFIX}{:x}",
            Sha256::digest(canonical.as_bytes())
        ))
    }
}

/// Identity of one event. Ordinal comes from rule precedence, never run order.
#[derive(Debug, Clone, Copy)]
pub struct EventIdentity<'a> {
    pub engine_version: &'a str,
    pub algorithm_version: &'a str,
    pub symbol: &'a str,
    pub timeframe: &'a str,
    pub data_revision: &'a str,
    pub config_hash: &'a str,
    pub source_module: Module,
    pub rule_id: &'a str,
    pub event_type: &'a str,
    pub observed_at: i64,
    pub known_at: i64,
    pub ordinal: u64,
}

/// Hash byte-length-prefixed UTF-8 fields in the exact event-v1 contract order.
pub fn stable_event_id(event: &EventIdentity<'_>) -> Result<String, ContractError> {
    for (path, value) in [
        ("engine_version", event.engine_version),
        ("algorithm_version", event.algorithm_version),
        ("symbol", event.symbol),
        ("timeframe", event.timeframe),
        ("data_revision", event.data_revision),
        ("config_hash", event.config_hash),
        ("rule_id", event.rule_id),
        ("event_type", event.event_type),
    ] {
        if value.trim().is_empty() {
            return Err(ContractError::new(
                ErrorCode::InvalidMetadata,
                format!("event.{path}"),
            ));
        }
    }
    let digest = event.config_hash.strip_prefix(CONFIG_HASH_PREFIX);
    if !digest.is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err(ContractError::new(
            ErrorCode::InvalidMetadata,
            "event.config_hash",
        ));
    }
    validate_timestamp(event.observed_at, "event.observed_at")?;
    validate_timestamp(event.known_at, "event.known_at")?;
    if event.observed_at > event.known_at {
        return Err(ContractError::new(
            ErrorCode::InvalidEvidence,
            "event.known_at",
        ));
    }
    let observed = event.observed_at.to_string();
    let known = event.known_at.to_string();
    let ordinal = event.ordinal.to_string();
    let mut hasher = Sha256::new();
    for field in [
        event.engine_version,
        event.algorithm_version,
        event.symbol,
        event.timeframe,
        event.data_revision,
        event.config_hash,
        event.source_module.as_str(),
        event.rule_id,
        event.event_type,
        &observed,
        &known,
        &ordinal,
    ] {
        hasher.update(field.len().to_string().as_bytes());
        hasher.update(b":");
        hasher.update(field.as_bytes());
    }
    Ok(format!("sha256:event-v1:{:x}", hasher.finalize()))
}
