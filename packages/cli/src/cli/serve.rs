use super::*;
use crate::{AddressArguments, BuildArgs, DioxusCrate, Platform};
use anyhow::Context;

/// Serve the project
#[derive(Clone, Debug, Default, Parser)]
#[command(group = clap::ArgGroup::new("release-incompatible").multiple(true).conflicts_with("release"))]
#[clap(name = "serve")]
pub(crate) struct ServeArgs {
    /// The arguments for the address the server will run on
    #[clap(flatten)]
    pub(crate) address: AddressArguments,

    /// Open the app in the default browser [default: true - unless cli settings are set]
    #[arg(long, default_missing_value="true", num_args=0..=1)]
    pub(crate) open: Option<bool>,

    /// Enable full hot reloading for the app [default: true - unless cli settings are set]
    #[clap(long, group = "release-incompatible")]
    pub(crate) hot_reload: Option<bool>,

    /// Configure always-on-top for desktop apps [default: true - unless cli settings are set]
    #[clap(long, default_missing_value = "true")]
    pub(crate) always_on_top: Option<bool>,

    /// Set cross-origin-policy to same-origin [default: false]
    #[clap(name = "cross-origin-policy")]
    #[clap(long)]
    pub(crate) cross_origin_policy: bool,

    /// Additional arguments to pass to the executable
    #[clap(long)]
    pub(crate) args: Vec<String>,

    /// Sets the interval in seconds that the CLI will poll for file changes on WSL.
    #[clap(long, default_missing_value = "2")]
    pub(crate) wsl_file_poll_interval: Option<u16>,

    /// Run the server in interactive mode
    #[arg(long, default_missing_value="true", num_args=0..=1, short = 'i')]
    pub(crate) interactive: Option<bool>,

    /// Arguments for the build itself
    #[clap(flatten)]
    pub(crate) build_arguments: BuildArgs,
}

impl ServeArgs {
    /// Start the tui, builder, etc by resolving the arguments and then running the actual top-level serve function
    pub(crate) async fn serve(mut self) -> Result<()> {
        let krate = DioxusCrate::new(&self.build_arguments.target_args)
            .context("Failed to load Dioxus workspace")?;

        // Enable hot reload.
        if self.hot_reload.is_none() {
            self.hot_reload = Some(krate.settings.always_hot_reload.unwrap_or(true));
        }

        // Open browser.
        if self.open.is_none() {
            self.open = Some(krate.settings.always_open_browser.unwrap_or_default());
        }

        // Set WSL file poll interval.
        if self.wsl_file_poll_interval.is_none() {
            self.wsl_file_poll_interval = Some(krate.settings.wsl_file_poll_interval.unwrap_or(2));
        }

        // Set always-on-top for desktop.
        if self.always_on_top.is_none() {
            self.always_on_top = Some(krate.settings.always_on_top.unwrap_or(true))
        }

        // Resolve the build arguments
        self.build_arguments.resolve(&krate)?;

        // Give us some space before we start printing things...
        println!();

        crate::serve::serve_all(self, krate).await
    }

    pub(crate) fn should_hotreload(&self) -> bool {
        self.hot_reload.unwrap_or(true)
    }

    pub(crate) fn build_args(&self) -> BuildArgs {
        self.build_arguments.clone()
    }

    pub(crate) fn is_interactive_tty(&self) -> bool {
        use crossterm::tty::IsTty;
        std::io::stdout().is_tty() && self.interactive.unwrap_or(true)
    }

    pub(crate) fn should_proxy_build(&self) -> bool {
        match self.build_arguments.platform() {
            Platform::Server => true,
            _ => self.build_arguments.fullstack,
        }
    }
}

impl std::ops::Deref for ServeArgs {
    type Target = BuildArgs;

    fn deref(&self) -> &Self::Target {
        &self.build_arguments
    }
}
