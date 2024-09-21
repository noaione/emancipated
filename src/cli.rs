use clap::{
    builder::{
        styling::{AnsiColor, Effects},
        Styles,
    },
    Parser, Subcommand,
};

pub(crate) type ExitCode = i32;

#[derive(Parser)]
#[command(name = "emancipated")]
#[command(bin_name = "emancipated")]
#[command(author, version = env!("CARGO_PKG_VERSION"), about, long_about = None, styles = cli_styles())]
#[command(propagate_version = true, disable_help_subcommand = true)]
pub(crate) struct EmancipatedCli {
    /// Increase message verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,
    /// Use proxy for all requests
    ///
    /// Format: `http(s)://<ip>:<port>` or `socks5://<ip>:<port>`.
    ///
    /// You can also add username and password to the URL like this:
    /// `http(s)://<username>:<password>@<ip>:<port>` or `socks5://<username>:<password>@<ip>:<port>`.
    #[arg(long)]
    pub(crate) proxy: Option<String>,

    /// Email/Account to use
    #[arg(short = 'a', long = "account", default_value = None)]
    pub(crate) account: Option<String>,

    #[command(subcommand)]
    pub(crate) command: EmancipatedCommands,
}

#[derive(Subcommand, Clone)]
pub(crate) enum EmancipatedCommands {
    /// Authenticate with your account.
    Auth {
        /// Email to use
        email: String,
        /// Password to use
        password: String,
    },
    /// Get an account information
    Account,
    /// See all the accounts you have authenticated with
    Accounts,
    /// Download specific volumes of a title
    Download {
        /// Slug of the title
        slug: String,
        /// Specify the volume(s) to download
        #[arg(short = 'n', long = "volume")]
        volume: u32,
        /// Enable parallel download
        #[arg(short = 'p', long = "parallel")]
        parallel: bool,
    },
    /// Get a title information including all the available volumes
    Info {
        /// Slug of the title
        slug: String,
    },
    /// Get your purchased titles
    Purchased,
    /// Search for a title
    Search {
        /// Search query
        query: String,
    },
}

fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Magenta.on_default() | Effects::BOLD | Effects::UNDERLINE)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::BrightCyan.on_default())
}
