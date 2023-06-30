use clap::{Parser, Subcommand};
use libra_types::legacy_types::app_cfg::AppCfg;
use libra_types::exports::Ed25519PrivateKey;
use std::path::PathBuf;
use crate::core::{proof, backlog};

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]

/// clap struct entry point for the tower cli
pub struct TowerCli {
    #[clap(subcommand)]
    command: TowerSub,
    /// If the node is offline and tower needs to run in local mode
    /// without querying chain
    #[clap(short,long)]
    local_mode: bool,
    /// The optional path to an alternate path besides $HOME/.0L
    #[clap(short,long)]
    config_file: Option<PathBuf>,
    /// nickname of the profile to use, if there is more than one. Defaults to first.
    #[clap(short,long)]
    profile: Option<String>,

    /// private key instead of using prompt. Warning: intended for tests only
    #[clap(short,long)]
    test_private_key: Option<String>,
}

#[derive(Subcommand)]
enum TowerSub {
  Backlog { 
    /// Just show the backlog of proofs not submitted
    #[clap(short,long)]
    show: bool 
  },
  Start,
  Test,
  Zero,
}

impl TowerCli {
    pub async fn run(&self) -> anyhow::Result<()>{
      let cli = TowerCli::parse();

      let mut app_cfg = AppCfg::load(cli.config_file)?;
      let pk = if let Some(pk_string) = self.test_private_key {
        Ed25519PrivateKey::from_str(&pk_string).ok()
      } else { None };
            
      match cli.command {
        TowerSub::Backlog { show } => {
          println!("backlog");
          if show {
            backlog::show_backlog(&app_cfg).await?;
          } else {
            backlog::process_backlog(&app_cfg, pk).await?;
          }
        },
        TowerSub::Start => {
          proof::mine_and_submit(&mut app_cfg, cli.local_mode).await?;
        },
        TowerSub::Test => {
          println!("test");

        },
        TowerSub::Zero => {
          proof::write_genesis(&app_cfg)?;
        },
      }
      
      Ok(())
    }
}