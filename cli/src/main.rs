pub mod cli;
use clap::Parser;
use cli::*;
use dg_xch_cli::wallet_commands::{create_cold_wallet, get_plotnft_ready_state, migrate_plot_nft};
use dg_xch_clients::rpc::full_node::FullnodeClient;
use dg_xch_core::blockchain::sized_bytes::Bytes32;
use simple_logger::SimpleLogger;
use std::io::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    SimpleLogger::new().env().init().unwrap_or_default();

    match cli.action {
        RootCommands::GetCoinRecord { .. } => {
            //Do Stuff Here
        }
        RootCommands::MovePlotNFT {
            target_pool,
            launcher_id,
            mnemonic,
            fee,
        } => {
            let host = cli.fullnode_host.unwrap_or("localhost".to_string());
            let client = FullnodeClient::new(
                &host,
                cli.fullnode_port.unwrap_or(8444),
                cli.ssl_path,
                &None,
            );
            migrate_plot_nft(
                &client,
                &target_pool,
                &Bytes32::from(launcher_id),
                &mnemonic,
                fee.unwrap_or_default(),
            )
            .await?
        }
        RootCommands::GetPlotnftState { launcher_id } => {
            let host = cli.fullnode_host.unwrap_or("localhost".to_string());
            let client = FullnodeClient::new(
                &host,
                cli.fullnode_port.unwrap_or(8444),
                cli.ssl_path,
                &None,
            );
            get_plotnft_ready_state(&client, &Bytes32::from(launcher_id))
                .await
                .map(|_| ())?
        }
        RootCommands::CreateWallet { action } => match action {
            WalletAction::WithNFT { .. } => {}
            WalletAction::Cold => create_cold_wallet()?,
        },
    }
    Ok(())
}
