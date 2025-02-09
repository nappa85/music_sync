use std::path::PathBuf;

use clap::Parser;

use tracing::error;

mod command;
mod error;
mod queue;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Collection root folder
    folder: PathBuf,
    /// Artist filter (can be specified multiple times)
    #[arg(short, long)]
    artist: Option<Vec<String>>,
    /// MusicBrainz API calls per second, default 1
    #[arg(short, long)]
    rate_limit: Option<usize>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    queue::init_queue(args.rate_limit.unwrap_or(1));
    // scan returns a list of futures, we need to poll it
    match queue::scan(args.folder, args.artist).await {
        Ok(mut js) => while js.join_next().await.is_some() {},
        Err(e) => error!("{e}"),
    }
}
