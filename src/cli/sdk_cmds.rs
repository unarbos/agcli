//! SDK-parity command handlers for generic compose/runtime/preflight surfaces.

use anyhow::{Context, Result};

use crate::chain::Client;
use crate::cli::helpers::{json_to_subxt_value, print_json, safe_rao};
use crate::cli::{ExtrinsicEraKind, OutputFormat};

fn ensure_non_empty(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{label} cannot be empty.");
    }
    Ok(())
}

fn parse_json_values(raw: &Option<String>, label: &str) -> Result<Vec<subxt::dynamic::Value>> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    let parsed: serde_json::Value = serde_json::from_str(raw)
        .with_context(|| format!("Invalid {label}: must be valid JSON"))?;
    let arr = parsed
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid {label}: expected a JSON array."))?;
    Ok(arr.iter().map(json_to_subxt_value).collect())
}

fn validate_mortal_period(mortality_blocks: u64) -> Result<()> {
    if !(4..=65536).contains(&mortality_blocks) {
        anyhow::bail!(
            "Invalid --mortality-blocks: {mortality_blocks}. For mortal era, value must be in [4, 65536]."
        );
    }
    if !mortality_blocks.is_power_of_two() {
        anyhow::bail!(
            "Invalid --mortality-blocks: {mortality_blocks}. For mortal era, value must be a power of two."
        );
    }
    Ok(())
}

pub async fn handle_compose_call(
    client: &Client,
    output: OutputFormat,
    pallet: &str,
    call: &str,
    args_json: &Option<String>,
) -> Result<()> {
    ensure_non_empty(pallet, "--pallet")?;
    ensure_non_empty(call, "--call")?;
    let fields = parse_json_values(args_json, "--args-json")?;
    let tx = subxt::dynamic::tx(pallet, call, fields);
    let encoded = client.subxt().tx().call_data(&tx).map_err(|e| {
        anyhow::anyhow!("Failed to encode call data for {}.{}: {}", pallet, call, e)
    })?;
    let call_hex = format!("0x{}", hex::encode(&encoded));

    if output.is_json() {
        print_json(&serde_json::json!({
            "pallet": pallet,
            "call": call,
            "call_data_hex": call_hex,
            "call_data_len": encoded.len(),
        }));
    } else {
        println!("Composed call: {}.{}", pallet, call);
        println!("  bytes: {}", encoded.len());
        println!("  hex:   {}", call_hex);
    }
    Ok(())
}

pub async fn handle_runtime_api_call(
    client: &Client,
    output: OutputFormat,
    api: &str,
    method: &str,
    params_json: &Option<String>,
    at_block: Option<u32>,
) -> Result<()> {
    ensure_non_empty(api, "--api")?;
    ensure_non_empty(method, "--method")?;
    let params = parse_json_values(params_json, "--params-json")?;
    let payload = subxt::dynamic::runtime_api_call(api, method, params);

    let (result, block_hash_str) = if let Some(block_num) = at_block {
        let block_hash = client.get_block_hash(block_num).await?;
        let result = client
            .subxt()
            .runtime_api()
            .at(block_hash)
            .call(payload)
            .await?;
        (result, Some(format!("{:?}", block_hash)))
    } else {
        let result = client
            .subxt()
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        (result, None)
    };

    let raw_hex = format!("0x{}", hex::encode(result.encoded()));
    let decoded = result.to_value();
    let decoded_json = match &decoded {
        Ok(value) => serde_json::to_value(value).unwrap_or_else(|_| {
            serde_json::Value::String("Decoded value is not JSON-serializable".to_string())
        }),
        Err(err) => serde_json::Value::String(format!("<decode-error: {}>", err)),
    };

    if output.is_json() {
        print_json(&serde_json::json!({
            "api": api,
            "method": method,
            "at_block": at_block,
            "block_hash": block_hash_str,
            "result_scale_hex": raw_hex,
            "result_decoded": decoded_json,
        }));
    } else {
        println!("Runtime API call: {}::{}", api, method);
        if let Some(block_num) = at_block {
            println!("  at block: {}", block_num);
        }
        println!("  raw: {}", raw_hex);
        println!("  decoded: {}", decoded_json);
    }
    Ok(())
}

pub fn handle_validate_extrinsic_params(
    output: OutputFormat,
    nonce: Option<u64>,
    era: ExtrinsicEraKind,
    tip: Option<f64>,
    mortality_blocks: Option<u64>,
) -> Result<()> {
    if let Some(n) = nonce {
        if n > u32::MAX as u64 {
            anyhow::bail!(
                "Invalid --nonce: {} is above max u32 value {}.",
                n,
                u32::MAX
            );
        }
    }

    let tip_tao = tip.unwrap_or(0.0);
    if !tip_tao.is_finite() || tip_tao < 0.0 {
        anyhow::bail!("Invalid --tip: must be a finite, non-negative TAO amount.");
    }
    let tip_rao = safe_rao(tip_tao);

    match era {
        ExtrinsicEraKind::Immortal => {
            if mortality_blocks.is_some() {
                anyhow::bail!("--mortality-blocks can only be used when --era mortal.");
            }
        }
        ExtrinsicEraKind::Mortal => {
            let blocks = mortality_blocks.ok_or_else(|| {
                anyhow::anyhow!("--mortality-blocks is required when --era mortal.")
            })?;
            validate_mortal_period(blocks)?;
        }
    }

    if output.is_json() {
        print_json(&serde_json::json!({
            "valid": true,
            "nonce": nonce,
            "era": match era {
                ExtrinsicEraKind::Immortal => "immortal",
                ExtrinsicEraKind::Mortal => "mortal",
            },
            "mortality_blocks": mortality_blocks,
            "tip_tao": tip_tao,
            "tip_rao": tip_rao,
        }));
    } else {
        println!("Extrinsic params are valid.");
        println!(
            "  nonce: {}",
            nonce.map_or_else(|| "auto".to_string(), |n| n.to_string())
        );
        println!(
            "  era: {}",
            match era {
                ExtrinsicEraKind::Immortal => "immortal".to_string(),
                ExtrinsicEraKind::Mortal => {
                    format!("mortal ({} blocks)", mortality_blocks.unwrap_or_default())
                }
            }
        );
        println!("  tip: {} TAO ({} RAO)", tip_tao, tip_rao);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_values_accepts_array() {
        let parsed = parse_json_values(&Some("[1,true,\"x\"]".to_string()), "--args-json").unwrap();
        assert_eq!(parsed.len(), 3);
    }

    #[test]
    fn parse_json_values_rejects_non_array() {
        let err = parse_json_values(&Some("{\"a\":1}".to_string()), "--args-json").unwrap_err();
        assert!(format!("{err}").contains("expected a JSON array"));
    }

    #[test]
    fn validate_mortal_period_requires_power_of_two() {
        let err = validate_mortal_period(63).unwrap_err();
        assert!(format!("{err}").contains("power of two"));
    }

    #[test]
    fn validate_extrinsic_params_mortal_requires_mortality() {
        let err = handle_validate_extrinsic_params(
            OutputFormat::Json,
            Some(1),
            ExtrinsicEraKind::Mortal,
            Some(0.0),
            None,
        )
        .unwrap_err();
        assert!(format!("{err}").contains("--mortality-blocks is required"));
    }
}
