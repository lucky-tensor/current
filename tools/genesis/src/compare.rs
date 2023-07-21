//! functions for comparing LegacyRecovery data to a genesis blob
//!
//! every day is like sunday
//! -- morrissey via github copilot
use crate::genesis_reader;
use crate::supply::Supply;
use anyhow;
use libra_types::gas_coin::SlowWalletBalance;
use zapatos_types::account_config::CoinStoreResource;
use zapatos_types::transaction::Transaction;
use libra_types::exports::AccountAddress;
use libra_types::legacy_types::legacy_address::LegacyAddress;
use libra_types::legacy_types::legacy_recovery::{LegacyRecovery, read_from_recovery_file};
use libra_types::ol_progress::OLProgress;
use zapatos_storage_interface::state_view::LatestDbStateCheckpointView;

use zapatos_state_view::account_with_state_view::AsAccountWithStateView;
use zapatos_types::account_view::AccountView;
use std::path::PathBuf;
use indicatif::{ProgressIterator, ProgressBar};

#[derive(Debug)]
/// struct for holding the results of a comparison
pub struct CompareError {
    /// index of LegacyRecover
    pub index: u64,
    /// user account
    pub account: Option<LegacyAddress>,
    /// balance difference [LegacyRecover]- [genesis blob]
    pub bal_diff: i64,
    /// error message
    pub message: String,
}
/// Compare the balances in a recovery file to the balances in a genesis blob.
pub fn compare_recovery_vec_to_genesis_tx(
    recovery: &[LegacyRecovery],
    genesis_transaction: &Transaction,
    supply: &Supply,
) -> Result<Vec<CompareError>, anyhow::Error> {
    // start an empty btree map
    let mut err_list: Vec<CompareError> = vec![];

    let pb = ProgressBar::new(1000)
    .with_style(OLProgress::spinner())
    .with_message("Test database from genesis.blob");
    pb.enable_steady_tick(core::time::Duration::from_millis(500));
    // iterate over the recovery file and compare balances
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&genesis_transaction)?;
    pb.finish_and_clear();

    recovery.iter()
    .progress_with_style(OLProgress::bar())
    .with_message("Comaparing new genesis to recovery json")
    .enumerate()
    .for_each(|(i, v)| {
        if v.account.is_none() {
            err_list.push(CompareError {
                index: i as u64,
                account: None,
                bal_diff: 0,
                message: "account is None".to_string(),
            }); // instead of balance, if there is an account that is None, we insert the index of the recovery file
            return;
        };

        if v.account.unwrap() == LegacyAddress::ZERO {
            return;
        };
        if v.balance.is_none() {
            err_list.push(CompareError {
                index: i as u64,
                account: v.account,
                bal_diff: 0,
                message: "recovery file balance is None".to_string(),
            });
            return;
        }

        let convert_address = AccountAddress::from_hex_literal(&v.account.unwrap().to_hex_literal()).expect("could not convert address types");

        let db_state_view = db_rw.reader.latest_state_checkpoint_view().unwrap();
        let account_state_view = db_state_view.as_account_with_state_view(&convert_address);

        if let Some(slow_legacy) = &v.slow_wallet {
            // let db_state_view = db_rw.reader.latest_state_checkpoint_view().unwrap();
            // let account_state_view = db_state_view.as_account_with_state_view(&convert_address);
            let on_chain_slow_wallet = account_state_view
              .get_move_resource::<SlowWalletBalance>()
              .expect("should have a slow wallet struct")
              .unwrap();

            let expected_unlocked = supply.split_factor * slow_legacy.unlocked as f64;

            if on_chain_slow_wallet.unlocked != expected_unlocked.trunc() as u64 {
              err_list.push(CompareError {
                  index: i as u64,
                  account: v.account,
                  bal_diff: on_chain_slow_wallet.unlocked as i64 - expected_unlocked as i64,
                  message: "unexpected slow wallet unlocked".to_string(),
              });
            }

        }

        if let Some(balance_legacy) = &v.balance {

            let on_chain_balance = account_state_view
              .get_move_resource::<CoinStoreResource>()
              .expect("should have a CoinStore resource for balance")
              .unwrap();

            let expected_balance: f64= supply.split_factor * balance_legacy.coin as f64;

            if on_chain_balance.coin() != expected_balance.trunc() as u64 {
              err_list.push(CompareError {
                  index: i as u64,
                  account: v.account,
                  bal_diff: on_chain_balance.coin() as i64 - expected_balance as i64,
                  message: "unexpected slow wallet unlocked".to_string(),
              });
            }

        }

        // CoinStoreResource


        // 753,614,948,274
        // 47,005,000,000
        // 706,609,948,274

        // dbg!(&b);
        // .get_coin_store_resource()
        // .unwrap()
        // .unwrap()
        // .coin()

        // let _account_state = get_account_state(&db_rw.reader, convert_address, None)
        //   .context("cannot read db for account state")
        //   .unwrap()
        //   .context("no account state found")
        //   .unwrap();

        // dbg!(&account_state);


        // let ap = make_access_path(convert_address, "ancestry", "Ancestry").unwrap();
        // let version = db_rw.reader.get_latest_version().unwrap();
        // let state_value = db_rw.reader.get_state_value_by_version(&StateKey::access_path(ap), version).unwrap().unwrap();
        // let ancestry: AncestryResource = bcs::from_bytes(state_value.bytes()).unwrap();

        // dbg!(&ancestry);
        // let val_state = match db_rw
        //     .reader
        //     .get_latest_account_state(v.account.expect("need an address"))
        // {
        //     Ok(Some(val_state)) => val_state,
        //     _ => {
        //         err_list.push(CompareError {
        //             index: i as u64,
        //             account: v.account,
        //             bal_diff: 0,
        //             message: "find account blob".to_string(),
        //         });
        //         return;
        //     }
        // };

        // let account_state = match AccountState::try_from(&val_state) {
        //     Ok(account_state) => account_state,
        //     _ => {
        //         err_list.push(CompareError {
        //             index: i as u64,
        //             account: v.account,
        //             bal_diff: 0,
        //             message: "parse account state".to_string(),
        //         });
        //         return;
        //     }
        // };

        // let bal = match account_state.get_balance_resources() {
        //     Ok(bal) => bal,
        //     _ => {
        //         err_list.push(CompareError {
        //             index: i as u64,
        //             account: v.account,
        //             bal_diff: 0,
        //             message: "get balance resource".to_string(),
        //         });
        //         return;
        //     }
        // };

        // let genesis_bal = match bal.iter().next() {
        //     Some((_, b)) => b.coin(),
        //     _ => {
        //         err_list.push(CompareError {
        //             index: i as u64,
        //             account: v.account,
        //             bal_diff: 0,
        //             message: "genesis resource is None".to_string(),
        //         });
        //         return;
        //     }
        // };

        // let recovery_bal = v.balance.as_ref().unwrap().coin();
        // if recovery_bal != genesis_bal {
        //     err_list.push(CompareError {
        //         index: i as u64,
        //         account: v.account,
        //         bal_diff: recovery_bal as i64 - genesis_bal as i64,
        //         message: "balance mismatch".to_string(),
        //     });
        // }
    });

    Ok(err_list)
}

/// Compare the balances in a recovery file to the balances in a genesis blob.
pub fn compare_json_to_genesis_blob(
    json_path: PathBuf,
    genesis_path: PathBuf,
    supply: &Supply,
) -> Result<Vec<CompareError>, anyhow::Error> {
    let recovery = read_from_recovery_file(&json_path);

    let gen_tx = genesis_reader::read_blob_to_tx(genesis_path)?;
    compare_recovery_vec_to_genesis_tx(&recovery,&gen_tx, supply)
}


// Check that the genesis validators are present in the genesis blob file, once we read the db.

// pub fn check_val_set(
//   expected_vals: Vec<AccountAddress>,
//   genesis_path: PathBuf,
// ) -> Result<(), anyhow::Error>{
//       let (db_rw, _) = read_db_and_compute_genesis(&genesis_path)?;

//       let root_blob = db_rw
//       .reader
//       .get_latest_account_state(AccountAddress::ZERO)?
//       .expect("no account state blob");

//       let root_state = AccountState::try_from(&root_blob)?;

//       let val_set = root_state.get_validator_set()?
//       .expect("no validator config state");

//       let addrs = val_set.payload()
//       // .iter()
//       .map(|v| {
//         // dbg!(&v);
//         *v.account_address()
//       })
//       .collect::<Vec<AccountAddress>>();

//       assert!(addrs.len() == expected_vals.len(), "validator set length mismatch");

//       for v in expected_vals {
//         assert!(addrs.contains(&v), "genesis does not contain validator");
//       }

//       Ok(())

// }

// fn get_balance(account: &AccountAddress, db: &DbReaderWriter) -> u64 {
//     let db_state_view = db.reader.latest_state_checkpoint_view().unwrap();
//     let account_state_view = db_state_view.as_account_with_state_view(account);
//     account_state_view
//         .get_coin_store_resource()
//         .unwrap()
//         .unwrap()
//         .coin()
// }