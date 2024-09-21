use std::sync::LazyLock;

use anstream::println;
use color_print::cformat;
use inquire::Select;

pub(crate) static IS_WIN_VT_SUPPORTED: LazyLock<bool> = LazyLock::new(|| {
    if ::supports_hyperlinks::on(::supports_hyperlinks::Stream::Stdout) {
        true
    } else {
        crate::win_term::check_windows_vt_support()
    }
});

#[derive(Clone, Debug)]
pub struct ConsoleChoice {
    /// The name of the choice (also the key)
    pub name: String,
    /// The value of the choice (the value that would be shown)
    pub value: String,
}

impl std::fmt::Display for ConsoleChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone)]
pub struct Terminal {
    debug: u8,
}

impl Terminal {
    fn new(debug: u8) -> Self {
        Self { debug }
    }

    /// Check if we in debug mode
    pub fn is_debug(&self) -> bool {
        self.debug > 0
    }

    /// Log info to terminal
    pub fn info(&self, msg: impl Into<String>) {
        println!(
            "{}",
            cformat!(" [<cyan,strong>INFO</cyan,strong>] {}", msg.into())
        )
    }

    /// Log warning to terminal
    pub fn warn(&self, msg: impl Into<String>) {
        println!(
            "{}",
            cformat!(" [<yellow,strong>WARN</yellow,strong>] {}", msg.into())
        )
    }

    /// Log error to terminal
    pub fn error(&self, msg: impl Into<String>) {
        println!(
            "{}",
            cformat!("[<red,strong>ERROR</red,strong>] {}", msg.into())
        )
    }

    /// Log to terminal
    pub fn log(&self, msg: impl Into<String>) {
        if self.debug >= 1 {
            println!(
                "{}",
                cformat!("  [<magenta,strong>LOG</magenta,strong>] {}", msg.into())
            )
        }
    }

    // pub fn trace(&self, msg: &str) {
    //     if self.debug >= 2 {
    //         println!("{}", cformat!("[<blue,strong>TRACE</blue,strong>] {}", msg))
    //     }
    // }

    /// Do a single choice prompt
    pub fn choice(&self, prompt: &str, choices: Vec<ConsoleChoice>) -> Option<ConsoleChoice> {
        let choice = Select::new(prompt, choices).prompt_skippable();

        match choice {
            Ok(choice) => choice,
            Err(_) => None,
        }
    }

    /// Stop the current spinner
    // pub fn stop_status(&mut self) {
    //     match self.current_spinner.as_mut() {
    //         Some(spinner) => {
    //             spinner.finish();
    //             self.current_spinner = None;
    //         }
    //         None => {}
    //     }
    // }

    pub fn make_progress(
        &self,
        len: u64,
        message: Option<impl Into<String>>,
    ) -> indicatif::ProgressBar {
        let progress = indicatif::ProgressBar::new(len);
        progress.enable_steady_tick(std::time::Duration::from_millis(120));
        progress.set_style(
            indicatif::ProgressStyle::with_template(
                "{spinner:.blue} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}",
            )
            .unwrap()
            .progress_chars("#>-")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "]),
        );
        let message: Option<String> = message.map(|m| m.into());
        let message = message.unwrap_or("Processing".to_string());
        progress.set_message(message);
        progress
    }
}

/// Get the root console instance
pub fn get_console(debug: u8) -> Terminal {
    Terminal::new(debug)
}

pub(crate) mod macros {
    /// Create a clickable link/text in terminal
    ///
    /// Ref: [`GitHub Gist`](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda)
    macro_rules! linkify {
        ($url:expr, $text:expr) => {
            if *$crate::term::IS_WIN_VT_SUPPORTED {
                format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", $url, $text)
            } else {
                $text.to_string()
            }
        };
        ($url:expr) => {
            linkify!($url, $url)
        };
    }

    pub(crate) use linkify;
}
