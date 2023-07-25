#![forbid(unsafe_code)]

use anyhow::Context;
use clap::Parser;
// use zapatos_crypto::HashValue;
use std::path::PathBuf;

use libra_framework::{
    builder::framework_generate_upgrade_proposal::{init_move_dir_wrapper, libra_compile_script},
    builder::framework_release_bundle::libra_generate_script_proposal_impl,
    release::ReleaseTarget,
};

use zapatos_framework::ReleaseBundle;
use zapatos_types::account_address::AccountAddress;

// use libra_framework::builder::release_config_ext;

#[derive(Parser)]
#[clap(name = "libra-framework", author, version, propagate_version = true)]
enum Commands {
    /// Creates a .mrb move framework release
    Release(GenesisRelease),
    /// Creates all artifacts for a network governance upgrade
    Upgrade(UpgradeRelease),
}

fn main() {
    let cmd: Commands = Commands::parse();
    let result = match cmd {
        Commands::Release(release) => release.execute(),
        Commands::Upgrade(release) => release.execute(),
    };
    if let Err(e) = result {
        eprintln!("error: {:#}", e);
        std::process::exit(1)
    }
}
#[derive(Debug, Parser)]
struct GenesisRelease {
    /// The release target. One of head, devnet, testnet, or mainnet. Notice the type
    /// of target determines what packages are included in the release. For example,
    /// some packages may be available in testnet, but aren't in mainnet.
    #[clap(long, default_value = "head")]
    target: ReleaseTarget,

    /// Remove the source code from the release package to shrink its size.
    #[clap(short, long)]
    without_source_code: bool,
}

impl GenesisRelease {
    fn execute(self) -> anyhow::Result<()> {
        self.target.create_release(!self.without_source_code, None)
    }
}

/// Creates all artifacts for a network governance upgrade
/// NOTE: this is an 0L reconstruction of vendor apis.
#[derive(Debug, Parser)]
struct UpgradeRelease {
    /// dir to save all the artifacts for the release.
    #[clap(short, long)]
    output_dir: PathBuf,
    /// directory of the framework source code
    /// usually ./framework
    #[clap(short, long)]
    framework_local_dir: PathBuf,

    /// provide a prebuilt release framework.mrb
    #[clap(short, long)]
    mrb_path: PathBuf,

    /// TODO: optionally pass a config file with the release config
    /// if there are parameter or raw script changes.
    #[clap(short, long)]
    release_config: Option<PathBuf>,


}

impl UpgradeRelease {
    fn execute(self) -> anyhow::Result<()> {
        // we are usually only interested in upgrading the framework
        // dbg!(&self.mrb_path);
      let script_name = "framework_upgrade";
      let package_dir = self.output_dir.join(script_name);
      // if let Some(path) = self.mrb_path {
            // assert!(path.exists(), "no .mrb bundle at this address {path:?}");

            println!("preparing upgrade Move package from prebuilt framework at: {:?}", &self.mrb_path);

            let bundle = ReleaseBundle::read(self.mrb_path.clone()).context("could not read a bundle release")?;


            dbg!(&package_dir);
            std::fs::create_dir_all(&package_dir)
                .context("could not create the output directory {new_path:?}")?;

            // TODO: rename this. init_move_package_with_local_framework
            init_move_dir_wrapper(
                package_dir.clone(),
                script_name,
                self.framework_local_dir.clone(),
            )?;

            let tx_script_filename = package_dir
                .join("sources")
                .join(&format!("{}.move", script_name));
            dbg!(&tx_script_filename);
            libra_generate_script_proposal_impl(&bundle, AccountAddress::ONE, tx_script_filename, None)?;
            dbg!("ok");

            println!("compiling script");
            let (bytes, hash) = libra_compile_script(&package_dir)?;

            std::fs::write(package_dir.join("script.mv"), bytes)?;
            std::fs::write(package_dir.join("script_sha3"), hash.to_string().as_bytes())?;

            println!("success: upgrade script built at: {:?}", package_dir);
            println!("hash: {:?}", hash);

        // else {

        //     let release_cfg = libra_release_cfg_default();
        //     match main_generate_proposals::run(release_cfg, package_dir, self.framework_local_dir) {
        //         Ok(_) => HashValue::random(),
        //         Err(e) => bail!("could not create releas build, message: {}", &e.to_string()),
        //     }
        // };

        Ok(())
    }
}
