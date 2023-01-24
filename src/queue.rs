use std::{path::PathBuf, time::Duration};

use futures_util::{stream::unfold, TryStreamExt};

use once_cell::sync::OnceCell;

use stream_throttle::{ThrottlePool, ThrottleRate, ThrottledStream};

use tokio::{
    fs::read_dir,
    sync::{mpsc, oneshot},
    task::JoinSet,
};

use tracing::{debug, error};

use crate::{command::Command, error::Error};

// queue must be accessible from everywhere, but we need an input config parameter, so it can't be Lazy
static QUEUE: OnceCell<mpsc::UnboundedSender<Command>> = OnceCell::new();

pub fn init_queue(rate_limit: usize) {
    let (tx, rx) = mpsc::unbounded_channel::<Command>();
    tokio::spawn(async move {
        let rate = ThrottleRate::new(rate_limit, Duration::new(1, 0));
        let pool = ThrottlePool::new(rate);
        let stream = unfold(rx, |mut rx| async move {
            let command = rx.recv().await?;
            Some((Ok(command), rx))
        })
        .throttle(pool);
        stream
            .try_for_each_concurrent(None, |command| async move { command.dispatch().await })
            .await
            .ok();
    });
    QUEUE.set(tx).unwrap();
}

// commodity function to not expose the static
pub fn get_queue() -> &'static mpsc::UnboundedSender<Command> {
    QUEUE.get().unwrap()
}

pub async fn scan(folder: PathBuf, artist: Option<String>) -> Result<JoinSet<()>, Error> {
    // collect folder names
    let mut dir = read_dir(&folder).await?;
    let mut artists = Vec::new();
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            if let Some(folder_name) = path.file_name().and_then(|s| s.to_str()) {
                if let Some(artist) = &artist {
                    if !artist.eq_ignore_ascii_case(folder_name) {
                        debug!("Filtering artist {}", artist);
                        continue;
                    }
                }
                artists.push(folder_name.to_owned());
            }
        }
    }

    // a big paged search query is potentially less calls than a call for every artist
    let (tx, mut rx) = mpsc::unbounded_channel();
    get_queue()
        .send(Command::Artists(artists.clone(), 0, tx))
        .map_err(|_| Error::ChannelClosed)?;
    let mut js = JoinSet::new();
    while let Some(res) = rx.recv().await {
        for artist in res? {
            if let Some(pos) = artists
                .iter()
                .position(|a| a.eq_ignore_ascii_case(&artist.name))
            {
                let mut path = folder.clone();
                path.push(artists.remove(pos));
                if path.exists() {
                    // spawn a new task instead of executing syncronously, this way we can achieve paralellism if an higher rate limit is used
                    js.spawn(async move {
                        let (tx, rx) = oneshot::channel();
                        if get_queue()
                            .send(Command::ExistingAlbum(path.clone(), artist.id, tx))
                            .is_ok()
                        {
                            match rx.await {
                                Ok(Ok(())) => {}
                                Ok(Err(e)) => error!("{e}"),
                                Err(e) => error!("{e}"),
                            }
                        }
                    });
                } else {
                    error!(
                        "Can't find folder for artist {}, probably you need to rename it",
                        artist.name
                    );
                }
            } else {
                debug!("Ignoring artist {}", artist.name)
            }
        }
    }

    // remaining artists
    if !artists.is_empty() {
        error!("Artists not found: {}", artists.join(", "));
    }

    Ok(js)
}
