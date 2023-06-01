use std::path::PathBuf;

use clap::{Parser, Subcommand};
use libra_genesis_tools::{wizard::GenesisWizard, parse_json};
use ol_genesis_tools::process_snaphot;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct GenesisCliArgs {
    #[command(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
    /// does testing things
    ExportDb {
        /// lists test values
        #[arg(short, long)]
        snapshot_path: PathBuf,

        #[arg(short, long)]
        output_path: Option<PathBuf>,
    },
    Wizard {
        /// choose a different home data folder for all node data.
        /// defaults to $HOME/.libra
        #[arg(long)]
        home_dir: Option<PathBuf>,

        /// if we should use a local mrb framework instead of the one from github. This is useful for testing.
        #[arg(short,long)]
        local_framework: bool,
    }
}

fn main() -> anyhow::Result<()>{
    let cli = GenesisCliArgs::parse();
    match cli.command {
        Some(Sub::ExportDb { snapshot_path , output_path}) => {
          let lr = process_snaphot::db_backup_into_recovery_struct(snapshot_path);
          lr.write_recovery_file(output_path);

        }
        Some(Sub::Wizard { home_dir, local_framework }) => {
            GenesisWizard::default().start_wizard(home_dir, local_framework)?;
        }
        _ => {}
    }

    // Continued program logic goes here...
    Ok(())
}
