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

pub fn get_queue() -> &'static mpsc::UnboundedSender<Command> {
    QUEUE.get().unwrap()
}

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

pub async fn scan(folder: PathBuf, artist: Option<String>) -> Result<JoinSet<()>, Error> {
    let mut dir = read_dir(folder).await?;
    let mut js = JoinSet::new();
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
                let folder_name = folder_name.to_owned();
                js.spawn(async move {
                    let (tx, rx) = oneshot::channel();
                    if get_queue().send(Command::Artist(folder_name, tx)).is_ok() {
                        match rx.await {
                            Ok(Ok(artists)) => {
                                for artist in artists {
                                    let (tx, rx) = oneshot::channel();
                                    if get_queue()
                                        .send(Command::Album(path.clone(), artist.id, 0, tx))
                                        .is_ok()
                                    {
                                        match rx.await {
                                            Ok(Ok(())) => {}
                                            Ok(Err(e)) => error!("Album error: {e}"),
                                            Err(e) => error!("Album error: {e}"),
                                        }
                                    }
                                }
                            }
                            Ok(Err(e)) => error!("Artist error: {e}"),
                            Err(e) => error!("Artist error: {e}"),
                        }
                    }
                });
            }
        }
    }
    Ok(js)
}
