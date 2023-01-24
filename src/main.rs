use std::path::PathBuf;

use clap::Parser;

use musicbrainz_rs::config;

use once_cell::sync::Lazy;

use tracing::error;

mod command;
mod error;
mod queue;

// a static str is needed
static USER_AGENT: Lazy<String> = Lazy::new(|| {
    format!(
        "{}/{} ( {} )",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY")
    )
});

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    folder: PathBuf,
    #[arg(short, long)]
    artist: Option<String>,
    #[arg(short, long)]
    rate_limit: Option<usize>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    config::set_user_agent(&USER_AGENT);
    queue::init_queue(args.rate_limit.unwrap_or(1));
    // scan returns a list of futures, we need to poll it
    match queue::scan(args.folder, args.artist).await {
        Ok(mut js) => while js.join_next().await.is_some() {},
        Err(e) => error!("{e}"),
    }
}
