use std::{io, path::PathBuf};

use clap::Parser;

use musicbrainz_rs::{
    entity::{artist::Artist, release::Release},
    Browse, Search,
};

use tokio::{fs::read_dir, task::JoinSet};

use tracing::{debug, error};

enum Error {
    Musicbrainz(musicbrainz_rs::Error),
}

impl From<musicbrainz_rs::Error> for Error {
    fn from(value: musicbrainz_rs::Error) -> Self {
        Error::Musicbrainz(value)
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    folder: PathBuf,
}

async fn scan(folder: PathBuf) -> io::Result<JoinSet<Result<(), Error>>> {
    let mut dir = read_dir(folder).await?;
    let mut set = JoinSet::new();
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            set.spawn(check_artist(path));
        }
    }
    Ok(set)
}

async fn check_artist(folder: PathBuf) -> Result<(), Error> {
    // fucking blocking apis
    tokio::task::spawn_blocking(move || {
        let folder_name = folder.file_name().unwrap().to_str().unwrap();
        let query = Artist::query_builder().name(folder_name).build();
        let res = Artist::search(query).execute()?;
        for artist in res.entities {
            if !artist.name.eq_ignore_ascii_case(folder_name) {
                debug!("Skipping artist {}", artist.name);
                continue;
            }
            let res = Release::browse().by_artist(&artist.id).execute()?;
            for release in res.entities {
                let mut subfolder = folder.clone();
                subfolder.push(format!(
                    "{} - {}",
                    release.date.unwrap_or_default().format("%Y"),
                    release.title
                ));
                if !subfolder.exists() {
                    error!("Missing album {}", subfolder.display());
                }
            }
        }
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let mut set = scan(args.folder).await.unwrap();
    while set.join_next().await.is_some() {}
}
