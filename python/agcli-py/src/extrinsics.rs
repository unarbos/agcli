use agcli::chain::subxt::ext::sp_core::sr25519;
use agcli::types::network::NetUid;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3_async_runtimes::tokio::future_into_py;

use crate::chain_data::{PyAlphaBalance, PySubnetIdentity};
use crate::client::PyClient;
use crate::errors::{map_error, ValidationError};
use crate::types::{parse_hash, PyBalance, PyNetUid};
use crate::wallet::PyWallet;

fn validation_error(message: impl Into<String>) -> PyErr {
    let message = message.into();
    Python::with_gil(|py| {
        let py_err = ValidationError::new_err(message);
        let value = py_err.value(py);
        let _ = value.setattr("code", agcli::error::exit_code::VALIDATION);
        py_err
    })
}

fn ensure_ss58(label: &str, ss58: &str) -> PyResult<()> {
    agcli::Client::ss58_to_account_id_pub(ss58)
        .map(|_| ())
        .map_err(|e| validation_error(format!("invalid {label} SS58 address: {e}")))
}

fn parse_netuid(value: &Bound<'_, PyAny>, field: &str) -> PyResult<NetUid> {
    if let Ok(netuid) = value.extract::<PyRef<'_, PyNetUid>>() {
        return Ok(netuid.inner);
    }
    let raw = value
        .extract::<i128>()
        .map_err(|_| validation_error(format!("{field} must be an integer or NetUid instance")))?;
    if raw < 0 {
        return Err(validation_error(format!("{field} cannot be negative")));
    }
    if raw > u16::MAX as i128 {
        return Err(validation_error(format!(
            "{field} {raw} exceeds maximum value {}",
            u16::MAX
        )));
    }
    Ok(NetUid(raw as u16))
}

fn parse_balance(value: &Bound<'_, PyAny>, field: &str) -> PyResult<agcli::Balance> {
    if let Ok(balance) = value.extract::<PyRef<'_, PyBalance>>() {
        return Ok(balance.inner());
    }
    let raw = value.extract::<i128>().map_err(|_| {
        validation_error(format!(
            "{field} must be a Balance or non-negative integer amount in rao"
        ))
    })?;
    if raw < 0 {
        return Err(validation_error(format!("{field} cannot be negative")));
    }
    if raw > u64::MAX as i128 {
        return Err(validation_error(format!(
            "{field} {raw} exceeds maximum value {}",
            u64::MAX
        )));
    }
    Ok(agcli::Balance::from_rao(raw as u64))
}

fn parse_alpha_balance(value: &Bound<'_, PyAny>, field: &str) -> PyResult<agcli::AlphaBalance> {
    if let Ok(balance) = value.extract::<PyRef<'_, PyAlphaBalance>>() {
        return Ok(balance.inner());
    }
    let raw = value.extract::<i128>().map_err(|_| {
        validation_error(format!(
            "{field} must be an AlphaBalance or non-negative integer amount in rao"
        ))
    })?;
    if raw < 0 {
        return Err(validation_error(format!("{field} cannot be negative")));
    }
    if raw > u64::MAX as i128 {
        return Err(validation_error(format!(
            "{field} {raw} exceeds maximum value {}",
            u64::MAX
        )));
    }
    Ok(agcli::AlphaBalance::from_raw(raw as u64))
}

fn parse_limit_price(value: u64) -> agcli::LimitPriceRao {
    agcli::LimitPriceRao::from_rao(value)
}

fn parse_hash_32(value: &Bound<'_, PyAny>, field: &str) -> PyResult<[u8; 32]> {
    let hash = parse_hash(value).map_err(|_| {
        validation_error(format!(
            "{field} must be 32-byte bytes or a 0x-prefixed 32-byte hex string"
        ))
    })?;
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_ref());
    Ok(out)
}

fn parse_u16_list(value: &Bound<'_, PyAny>, field: &str) -> PyResult<Vec<u16>> {
    let raw = value
        .extract::<Vec<i128>>()
        .map_err(|_| validation_error(format!("{field} must be a list of integers")))?;
    let mut parsed = Vec::with_capacity(raw.len());
    for (index, item) in raw.into_iter().enumerate() {
        if !(0..=u16::MAX as i128).contains(&item) {
            return Err(validation_error(format!(
                "{field}[{index}]={item} is outside the u16 range"
            )));
        }
        parsed.push(item as u16);
    }
    Ok(parsed)
}

fn wallet_hotkey_ss58(wallet: &agcli::Wallet) -> PyResult<String> {
    wallet.hotkey_ss58().map(str::to_string).ok_or_else(|| {
        validation_error("wallet hotkey SS58 is unavailable; call wallet.load_hotkey(...) first")
    })
}

fn wallet_coldkey_pair(wallet: &agcli::Wallet) -> PyResult<sr25519::Pair> {
    wallet.coldkey().map(|pair| pair.clone()).map_err(map_error)
}

fn wallet_hotkey_pair(wallet: &agcli::Wallet) -> PyResult<sr25519::Pair> {
    wallet.hotkey().map(|pair| pair.clone()).map_err(map_error)
}

fn apply_call_overrides(
    client: &mut agcli::Client,
    dry_run: bool,
    finalization_timeout: Option<u64>,
    mortality_blocks: Option<u64>,
) -> (bool, u64, u64) {
    let previous = (
        client.is_dry_run(),
        client.finalization_timeout(),
        client.mortality_blocks(),
    );
    if dry_run {
        client.set_dry_run(true);
    }
    if let Some(timeout) = finalization_timeout {
        client.set_finalization_timeout(timeout);
    }
    if let Some(blocks) = mortality_blocks {
        client.set_mortality_blocks(blocks);
    }
    previous
}

fn restore_call_overrides(client: &mut agcli::Client, previous: (bool, u64, u64)) {
    client.set_dry_run(previous.0);
    client.set_finalization_timeout(previous.1);
    client.set_mortality_blocks(previous.2);
}

#[pymethods]
impl PyClient {
    #[pyo3(signature = (wallet, dest_ss58, amount, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn transfer<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        dest_ss58: String,
        amount: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        ensure_ss58("destination", &dest_ss58)?;
        let amount = parse_balance(&amount, "amount")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.transfer(&pair, &dest_ss58, amount).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, dest_ss58, keep_alive=false, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn transfer_all<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        dest_ss58: String,
        keep_alive: bool,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        ensure_ss58("destination", &dest_ss58)?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.transfer_all(&pair, &dest_ss58, keep_alive).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, amount, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn add_stake<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let amount = parse_balance(&amount, "amount")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .add_stake_mev(&pair, &hotkey_ss58, netuid, amount, true)
                    .await
            } else {
                client.add_stake(&pair, &hotkey_ss58, netuid, amount).await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, amount, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn remove_stake<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let amount = parse_alpha_balance(&amount, "amount")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .remove_stake_mev(&pair, &hotkey_ss58, netuid, amount, true)
                    .await
            } else {
                client
                    .remove_stake(&pair, &hotkey_ss58, netuid, amount)
                    .await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, amount, limit_price, allow_partial=true, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn add_stake_limit<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        limit_price: u64,
        allow_partial: bool,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let amount = parse_balance(&amount, "amount")?;
        let limit_price = parse_limit_price(limit_price);
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .add_stake_limit_mev(
                        &pair,
                        &hotkey_ss58,
                        netuid,
                        amount,
                        limit_price,
                        allow_partial,
                        true,
                    )
                    .await
            } else {
                client
                    .add_stake_limit(
                        &pair,
                        &hotkey_ss58,
                        netuid,
                        amount,
                        limit_price,
                        allow_partial,
                    )
                    .await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, amount, limit_price, allow_partial=true, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn remove_stake_limit<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        limit_price: u64,
        allow_partial: bool,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let amount = parse_alpha_balance(&amount, "amount")?;
        let limit_price = parse_limit_price(limit_price);
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .remove_stake_limit_mev(
                        &pair,
                        &hotkey_ss58,
                        netuid,
                        amount,
                        limit_price,
                        allow_partial,
                        true,
                    )
                    .await
            } else {
                client
                    .remove_stake_limit(
                        &pair,
                        &hotkey_ss58,
                        netuid,
                        amount,
                        limit_price,
                        allow_partial,
                    )
                    .await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, from_netuid, to_netuid, amount, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn move_stake<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        from_netuid: Bound<'_, PyAny>,
        to_netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let from_netuid = parse_netuid(&from_netuid, "from_netuid")?;
        let to_netuid = parse_netuid(&to_netuid, "to_netuid")?;
        let amount = parse_alpha_balance(&amount, "amount")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .move_stake_mev(
                        &pair,
                        &hotkey_ss58,
                        &hotkey_ss58,
                        from_netuid,
                        to_netuid,
                        amount,
                        true,
                    )
                    .await
            } else {
                client
                    .move_stake(&pair, &hotkey_ss58, from_netuid, to_netuid, amount)
                    .await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, from_netuid, to_netuid, amount, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn swap_stake<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        from_netuid: Bound<'_, PyAny>,
        to_netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let from_netuid = parse_netuid(&from_netuid, "from_netuid")?;
        let to_netuid = parse_netuid(&to_netuid, "to_netuid")?;
        let amount = parse_alpha_balance(&amount, "amount")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .swap_stake_mev(&pair, &hotkey_ss58, from_netuid, to_netuid, amount, true)
                    .await
            } else {
                client
                    .swap_stake(&pair, &hotkey_ss58, from_netuid, to_netuid, amount)
                    .await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, from_netuid, to_netuid, amount, limit_price, allow_partial=true, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn swap_stake_limit<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        from_netuid: Bound<'_, PyAny>,
        to_netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        limit_price: u64,
        allow_partial: bool,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let from_netuid = parse_netuid(&from_netuid, "from_netuid")?;
        let to_netuid = parse_netuid(&to_netuid, "to_netuid")?;
        let amount = parse_alpha_balance(&amount, "amount")?;
        let limit_price = parse_limit_price(limit_price);
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .swap_stake_limit_mev(
                        &pair,
                        &hotkey_ss58,
                        from_netuid,
                        to_netuid,
                        amount,
                        limit_price,
                        allow_partial,
                        true,
                    )
                    .await
            } else {
                client
                    .swap_stake_limit(
                        &pair,
                        &hotkey_ss58,
                        from_netuid,
                        to_netuid,
                        amount,
                        limit_price,
                        allow_partial,
                    )
                    .await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, dest_ss58, from_netuid, to_netuid, amount, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn transfer_stake<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        dest_ss58: String,
        from_netuid: Bound<'_, PyAny>,
        to_netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        ensure_ss58("destination", &dest_ss58)?;
        let from_netuid = parse_netuid(&from_netuid, "from_netuid")?;
        let to_netuid = parse_netuid(&to_netuid, "to_netuid")?;
        let amount = parse_alpha_balance(&amount, "amount")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .transfer_stake_mev(
                        &pair,
                        &dest_ss58,
                        &hotkey_ss58,
                        from_netuid,
                        to_netuid,
                        amount,
                        true,
                    )
                    .await
            } else {
                client
                    .transfer_stake(
                        &pair,
                        &dest_ss58,
                        &hotkey_ss58,
                        from_netuid,
                        to_netuid,
                        amount,
                    )
                    .await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn unstake_all<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.unstake_all(&pair, &hotkey_ss58).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn unstake_all_alpha<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.unstake_all_alpha(&pair, &hotkey_ss58).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, amount, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn recycle_alpha<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let amount = parse_alpha_balance(&amount, "amount")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .recycle_alpha_mev(&pair, &hotkey_ss58, netuid, amount, true)
                    .await
            } else {
                client
                    .recycle_alpha(&pair, &hotkey_ss58, netuid, amount)
                    .await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, amount, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn burn_alpha<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        amount: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let amount = parse_alpha_balance(&amount, "amount")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = if mev {
                client
                    .burn_alpha_mev(&pair, &hotkey_ss58, amount, netuid, true)
                    .await
            } else {
                client.burn_alpha(&pair, &hotkey_ss58, amount, netuid).await
            };
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, subnets, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn claim_root<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        subnets: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let subnets = parse_u16_list(&subnets, "subnets")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.claim_root(&pair, &subnets).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, uids, values, version_key, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn set_weights<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        uids: Bound<'_, PyAny>,
        values: Bound<'_, PyAny>,
        version_key: u64,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let uids = parse_u16_list(&uids, "uids")?;
        let values = parse_u16_list(&values, "values")?;
        if uids.len() != values.len() {
            return Err(validation_error(format!(
                "uids and values length mismatch: {} != {}",
                uids.len(),
                values.len()
            )));
        }
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_hotkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client
                .set_weights(&pair, netuid, &uids, &values, version_key)
                .await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, commit_hash, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn commit_weights<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        commit_hash: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let commit_hash = parse_hash_32(&commit_hash, "commit_hash")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_hotkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.commit_weights(&pair, netuid, commit_hash).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, uids, values, salt, version_key, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn reveal_weights<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        uids: Bound<'_, PyAny>,
        values: Bound<'_, PyAny>,
        salt: Bound<'_, PyAny>,
        version_key: u64,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let uids = parse_u16_list(&uids, "uids")?;
        let values = parse_u16_list(&values, "values")?;
        let salt = parse_u16_list(&salt, "salt")?;
        if uids.len() != values.len() {
            return Err(validation_error(format!(
                "uids and values length mismatch: {} != {}",
                uids.len(),
                values.len()
            )));
        }
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_hotkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client
                .reveal_weights(&pair, netuid, &uids, &values, &salt, version_key)
                .await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn register_network<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.register_network(&pair, &hotkey_ss58).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn burned_register<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.burned_register(&pair, netuid, &hotkey_ss58).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, block_number, nonce, work, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn pow_register<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        block_number: u64,
        nonce: u64,
        work: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let work = parse_hash_32(&work, "work")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client
                .pow_register(&pair, netuid, &hotkey_ss58, block_number, nonce, work)
                .await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn root_register<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey_ss58 = wallet_hotkey_ss58(&wallet)?;
            ensure_ss58("wallet hotkey", &hotkey_ss58)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.root_register(&pair, &hotkey_ss58).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn dissolve_network<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.dissolve_network(&pair, netuid).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, delegate_ss58, proxy_type, delay=0, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn add_proxy<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        delegate_ss58: String,
        proxy_type: String,
        delay: u32,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        ensure_ss58("delegate", &delegate_ss58)?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client
                .add_proxy(&pair, &delegate_ss58, &proxy_type, delay)
                .await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, delegate_ss58, proxy_type, delay=0, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn remove_proxy<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        delegate_ss58: String,
        proxy_type: String,
        delay: u32,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        ensure_ss58("delegate", &delegate_ss58)?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client
                .remove_proxy(&pair, &delegate_ss58, &proxy_type, delay)
                .await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, proxy_type, delay=0, index=0, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn create_pure_proxy<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        proxy_type: String,
        delay: u32,
        index: u16,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client
                .create_pure_proxy(&pair, &proxy_type, delay, index)
                .await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, spawner_ss58, proxy_type, index, height, ext_index, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn kill_pure_proxy<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        spawner_ss58: String,
        proxy_type: String,
        index: u16,
        height: u32,
        ext_index: u32,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        ensure_ss58("spawner", &spawner_ss58)?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client
                .kill_pure_proxy(&pair, &spawner_ss58, &proxy_type, index, height, ext_index)
                .await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, hotkey_ss58=None, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn try_associate_hotkey<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        hotkey_ss58: Option<String>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let hotkey = hotkey_ss58.unwrap_or_else(|| {
                wallet
                    .hotkey_ss58()
                    .map(str::to_string)
                    .unwrap_or_else(String::new)
            });
            if hotkey.is_empty() {
                return Err(validation_error(
                    "hotkey_ss58 is required when wallet has no loaded hotkey address",
                ));
            }
            ensure_ss58("hotkey", &hotkey)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.try_associate_hotkey(&pair, &hotkey).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, new_coldkey_ss58, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn schedule_swap_coldkey<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        new_coldkey_ss58: String,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        ensure_ss58("new_coldkey", &new_coldkey_ss58)?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.announce_swap_coldkey(&pair, &new_coldkey_ss58).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, old_hotkey_ss58, new_hotkey_ss58, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn swap_hotkey<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        old_hotkey_ss58: String,
        new_hotkey_ss58: String,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        ensure_ss58("old_hotkey", &old_hotkey_ss58)?;
        ensure_ss58("new_hotkey", &new_hotkey_ss58)?;
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client
                .swap_hotkey(&pair, &old_hotkey_ss58, &new_hotkey_ss58)
                .await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }

    #[pyo3(signature = (wallet, netuid, identity, *, wait=true, mev=true, dry_run=false, finalization_timeout=None, mortality_blocks=None))]
    fn set_subnet_identity<'py>(
        &self,
        py: Python<'py>,
        wallet: PyRef<'_, PyWallet>,
        netuid: Bound<'_, PyAny>,
        identity: Bound<'_, PyAny>,
        wait: bool,
        mev: bool,
        dry_run: bool,
        finalization_timeout: Option<u64>,
        mortality_blocks: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = wait;
        let _ = mev;
        let netuid = parse_netuid(&netuid, "netuid")?;
        let identity = if let Ok(identity) = identity.extract::<PyRef<'_, PySubnetIdentity>>() {
            identity.inner_clone()
        } else {
            pythonize::depythonize::<agcli::types::chain_data::SubnetIdentity>(&identity)
                .map_err(|e| validation_error(format!("invalid subnet identity payload: {e}")))?
        };
        let client = self.shared_client();
        let wallet = wallet.shared_wallet();
        future_into_py(py, async move {
            let mut client = client.lock().await;
            let wallet = wallet.lock().await;
            let pair = wallet_coldkey_pair(&wallet)?;
            let previous =
                apply_call_overrides(&mut client, dry_run, finalization_timeout, mortality_blocks);
            let result = client.set_subnet_identity(&pair, netuid, &identity).await;
            restore_call_overrides(&mut client, previous);
            result.map_err(map_error)
        })
    }
}
