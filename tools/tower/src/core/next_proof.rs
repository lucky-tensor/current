//! next proof


use crate::core::proof_preimage;

use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};

use libra_types::legacy_types::{
  app_cfg::AppCfg,
  block::{GENESIS_VDF_ITERATIONS, GENESIS_VDF_SECURITY_PARAM, VDFProof},
  vdf_difficulty::VDFDifficulty,
};

use libra_query::{get_client, account_queries, chain_queries};

use zapatos_sdk::crypto::HashValue;

/// container for the next proof parameters to be fed to VDF prover.
#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct NextProof {
    ///
    pub diff: VDFDifficulty,
    ///
    pub next_height: u64,
    ///
    pub preimage: Vec<u8>,
}

impl NextProof {
    /// create a genesis proof
    pub fn genesis_proof(config: &AppCfg) -> Self {
      // NOTE: can't set defautlsin VDFDifficulty::default() because of circular dependency
        let mut diff = VDFDifficulty::default();

        diff.difficulty = GENESIS_VDF_ITERATIONS.clone();
        diff.security = GENESIS_VDF_SECURITY_PARAM.clone();
        diff.prev_diff = GENESIS_VDF_ITERATIONS.clone();
        diff.prev_sec = GENESIS_VDF_SECURITY_PARAM.clone();

        NextProof {
            diff,
            next_height: 0,
            preimage: proof_preimage::genesis_preimage(config),
        }
    }
}
/// return the VDF difficulty expected and the next tower height
pub fn get_next_proof_params_from_local(config: &AppCfg) -> Result<NextProof, Error> {
    // get the location of this miner's blocks
    let mut blocks_dir = config.workspace.node_home.clone();
    blocks_dir.push(&config.workspace.block_dir);
    let (current_local_block, _) = VDFProof::get_highest_block(&blocks_dir)?;
    let diff = VDFDifficulty {
        difficulty: current_local_block.difficulty(),
        security: current_local_block.security.unwrap(),
        prev_diff: current_local_block.difficulty(),
        prev_sec: current_local_block.security.unwrap(),
    };
    Ok(NextProof {
        diff,
        next_height: current_local_block.height + 1,
        preimage: HashValue::sha3_256_of(&current_local_block.proof).to_vec(),
    })
}

/// query the chain for parameters to use in the next VDF proof.
/// includes global parameters for difficulty
/// and individual parameters like tower height and the preimage (previous proof hash)
pub async fn get_next_proof_from_chain(
    app_cfg: &AppCfg,
    // // client: DiemClient,
    // swarm_path: Option<PathBuf>,
) -> Result<NextProof, Error> {
    // todo!();
    let (client, _) = get_client::find_good_upstream(app_cfg).await?;
    // // dbg!("pick_client");
    // // let client = pick_client(swarm_path.clone(), config)?;

    // // dbg!("get user tower state");
    // let mut n = Node::new(client, config, swarm_path.is_some());

    let (difficulty, security) = chain_queries::get_tower_difficulty(&client).await?;

    // get user's state
    let p = match account_queries::get_tower_state(&client, app_cfg.profile.account).await {
      Ok(ts) => {
        NextProof {
            diff: VDFDifficulty{
              difficulty,
              security,
              prev_diff: 0, // not relevant off chain
              prev_sec: 0, // not relevant off chain
          },
            next_height: ts.verified_tower_height + 1, // add one for next
            preimage: ts.previous_proof_hash,
        }
      }
      _ => bail!("cannot get tower resource for account")

    };

    Ok(p)


    // n.refresh_onchain_state();
    // // TODO: we are picking Client twice
    // let diff = get_difficulty_from_chain(&n)?;
  
    // // get the user's tower state from chain.
    // let ts = n.client
    //   .get_account_state(config.profile.account)?
    //   .get_miner_state()?;

    //   if let Some(t) = tower_view {
    //     Ok()
    //   } else {
    //     bail!("cannot get tower resource for account")
    // }
}

// /// Get the VDF difficulty from chain.
// pub fn get_difficulty_from_chain(n: &Node) -> anyhow::Result<VDFDifficulty> {

//     if let Some(a) = &n.chain_state {

//         if let Some(diff) = a.get_tower_params()? {
//             return Ok(diff);
//         }
//         bail!("could not get this epoch's VDF params from chain.")
//     }
//     bail!("could not get account state for 0x0")
// }

