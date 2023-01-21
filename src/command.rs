use std::path::PathBuf;

use musicbrainz_rs::{
    entity::{
        artist::{Artist, ArtistSearchQuery},
        release_group::ReleaseGroup,
    },
    Browse, Search,
};

use tokio::sync::oneshot;

use crate::{error::Error, queue::get_queue};

#[derive(Debug)]
pub enum Command {
    Artist(String, oneshot::Sender<Result<Vec<Artist>, Error>>),
    Album(PathBuf, String, u16, oneshot::Sender<Result<(), Error>>),
}

impl Command {
    pub async fn dispatch(self) -> Result<(), Error> {
        match self {
            Command::Artist(name, reply) => {
                let query = ArtistSearchQuery::query_builder().artist(&name).build();
                match Artist::search(query).execute().await {
                    Ok(res) => reply
                        .send(Ok(res
                            .entities
                            .into_iter()
                            .filter(|artist| artist.name.eq_ignore_ascii_case(&name))
                            .collect()))
                        .map_err(|_| Error::ChannelClosed)?,
                    Err(e) => reply
                        .send(Err(e.into()))
                        .map_err(|_| Error::ChannelClosed)?,
                }
            }
            Command::Album(folder, artist_id, offset, reply) => {
                match ReleaseGroup::browse()
                    .by_artist(&artist_id)
                    .limit(100)
                    .offset(offset)
                    .execute()
                    .await
                {
                    Ok(res) => {
                        for release in res.entities {
                            let mut subfolder = folder.clone();
                            subfolder.push(format!(
                                "{} - {}",
                                release.first_release_date.unwrap_or_default().format("%Y"),
                                release.title
                            ));
                            if !subfolder.exists() {
                                println!("Missing album {}", subfolder.display());
                            }
                        }

                        if offset as i32 + 100 < res.count {
                            get_queue()
                                .send(Command::Album(folder, artist_id, offset + 100, reply))
                                .map_err(|_| Error::ChannelClosed)?;
                        } else {
                            reply.send(Ok(())).map_err(|_| Error::ChannelClosed)?;
                        }
                    }
                    Err(e) => reply
                        .send(Err(e.into()))
                        .map_err(|_| Error::ChannelClosed)?,
                }
            }
        }
        Ok(())
    }
}
