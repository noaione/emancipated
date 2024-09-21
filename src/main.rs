use clap::Parser;
use cli::{EmancipatedCli, ExitCode};
use client::ClientError;
use config::{get_user_path, select_single_account};

mod cli;
mod client;
mod commands;
mod config;
mod image;
mod kp;
mod models;
mod term;
mod win_term;

#[tokio::main]
async fn main() {
    let cli = EmancipatedCli::parse();
    let exit_code = entrypoint(cli).await.unwrap();

    std::process::exit(exit_code);
}

async fn entrypoint(cli: EmancipatedCli) -> Result<ExitCode, ClientError> {
    let t = term::get_console(cli.verbose);

    let parsed_proxy = match cli.proxy {
        Some(proxy) => match reqwest::Proxy::all(proxy) {
            Ok(proxy) => Some(proxy),
            Err(e) => {
                t.warn(&format!("Unable to parse proxy: {}", e));
                return Ok(1);
            }
        },
        None => None,
    };

    let user_home = get_user_path();
    if !user_home.exists() {
        std::fs::create_dir_all(&user_home).unwrap();
    }

    let early_exit = match cli.command.clone() {
        cli::EmancipatedCommands::Auth { email, password } => {
            Some(commands::accounts::accounts_auth(email, password, parsed_proxy.clone(), &t).await)
        }
        cli::EmancipatedCommands::Accounts => Some(commands::accounts::accounts_all(&t).await),
        _ => None,
    };

    if let Some(exit_code) = early_exit {
        std::process::exit(exit_code);
    }

    let config = select_single_account(cli.account.as_deref(), &t);
    let mut config = match config {
        Some(config) => config,
        None => {
            t.warn("No account selected!");
            std::process::exit(1);
        }
    };

    let mut client = client::Client::new(&mut config, parsed_proxy)?;

    let exit_code = match cli.command {
        cli::EmancipatedCommands::Auth {
            email: _,
            password: _,
        } => 0,
        cli::EmancipatedCommands::Account => {
            commands::accounts::accounts_info(&mut client, &t).await
        }
        cli::EmancipatedCommands::Accounts => 0,
        cli::EmancipatedCommands::Download {
            slug,
            volume,
            parallel,
        } => commands::download::manga_download(&mut client, &t, slug, volume, parallel).await,
        cli::EmancipatedCommands::Info { slug } => {
            commands::manga::manga_info(&mut client, &t, slug).await
        }
        cli::EmancipatedCommands::Purchased => 1,
        cli::EmancipatedCommands::Search { query } => {
            commands::manga::manga_search(&mut client, &t, query).await
        }
    };

    Ok(exit_code)
}
