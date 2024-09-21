use color_print::cformat;

use crate::{cli::ExitCode, client::Client, config::save_config};

pub(crate) async fn accounts_auth(
    email: impl Into<String>,
    password: impl Into<String>,
    proxy: Option<reqwest::Proxy>,
    console: &crate::term::Terminal,
) -> ExitCode {
    let email: String = email.into();
    console.info(cformat!("Logging in as <m,s>{}</>...", &email));
    match crate::client::Client::login(&email, password, proxy).await {
        Ok(account) => {
            let mut config = crate::config::Config::from(&account);

            console.info(cformat!("Logged in as <m,s>{}</>", account.email));

            match config.generate_key_pair() {
                Ok(_) => {
                    console.info("Generated RSA key pair, saving config...");
                    save_config(&config);
                    0
                }
                Err(e) => {
                    console.error(cformat!("Failed to generate RSA key pair: <r,s>{}</>", e));
                    1
                }
            }
        }
        Err(e) => {
            console.error(cformat!("Failed to login: <r,s>{}</>", e));
            1
        }
    }
}

pub(crate) async fn accounts_info(
    client: &mut Client,
    console: &crate::term::Terminal,
) -> ExitCode {
    let account = client.get_config_owned();
    console.info(cformat!(
        "Fetching user <m,s>{}</> information",
        account.email()
    ));

    match client.get_user().await {
        Ok(user_info) => {
            save_config(client.get_config());

            console.info(cformat!("Account info for <m,s>{}</>:", account.email()));
            console.info(cformat!("  - <s>ID</>: {}", user_info.user.id));
            console.info(cformat!("  - <s>Email</>: {}", account.email()));
            console.info(cformat!(
                "  - <s>Coins</>: {}<y,s>c</>",
                user_info.user.coins
            ));
            if let Some(pronouns) = user_info.profile.pronouns {
                if !pronouns.is_empty() {
                    console.info(cformat!("  - <s>Pronouns</>: {}", pronouns));
                }
            }
            if let Some(dob) = user_info.profile.dob {
                if !dob.is_empty() {
                    console.info(cformat!("  - <s>Date of Birth</>: {}", dob));
                }
            }

            0
        }
        Err(e) => {
            console.error(cformat!("Failed to fetch user info: <r,s>{}</>", e));
            1
        }
    }
}

pub(crate) async fn accounts_all(console: &crate::term::Terminal) -> ExitCode {
    let all_configs = crate::config::find_any_config();

    match all_configs.len() {
        0 => {
            console.warn("No accounts found!");

            1
        }
        _ => {
            console.info(cformat!("Found {} accounts:", all_configs.len()));
            for (i, c) in all_configs.iter().enumerate() {
                console.info(cformat!("{:02}. <s>{}</>", i + 1, c.email()));
            }

            0
        }
    }
}
