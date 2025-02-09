use std::path::PathBuf;

use musicbrainz_rs::{
    entity::{
        artist::{Artist, ArtistSearchQuery},
        release_group::{ReleaseGroup, ReleaseGroupSearchQuery},
    },
    Browse, Search,
};

use tokio::{
    fs::read_dir,
    sync::{mpsc, oneshot},
};

use crate::{error::Error, queue::get_queue};

#[derive(Debug)]
pub enum Command {
    Artists(
        Vec<String>,
        u16,
        mpsc::UnboundedSender<Result<Vec<Artist>, Error>>,
    ),
    ExistingAlbum(PathBuf, String, oneshot::Sender<Result<(), Error>>),
    MissingAlbum(
        PathBuf,
        Vec<String>,
        String,
        u16,
        oneshot::Sender<Result<(), Error>>,
    ),
}

impl Command {
    pub async fn dispatch(self) -> Result<(), Error> {
        match self {
            Command::Artists(names, mut offset, reply) => {
                let mut query = ArtistSearchQuery::query_builder();
                let mut temp = &mut query;
                let mut first = true;
                for name in &names {
                    temp = if first {
                        first = false;
                        temp.artist(name.as_str())
                    } else {
                        temp.or().artist(name.as_str())
                    };
                }
                match Artist::search(query.build())
                    .offset(offset)
                    .limit(100)
                    .execute()
                    .await
                {
                    Ok(res) => {
                        offset += res.entities.len() as u16;
                        reply
                            .send(Ok(res
                                .entities
                                .into_iter()
                                .collect()))
                            .map_err(|_| Error::ChannelClosed)?;
                        if (offset as i32) < res.count {
                            get_queue()
                                .send(Command::Artists(names, offset, reply))
                                .map_err(|_| Error::ChannelClosed)?;
                        }
                    }
                    Err(e) => reply
                        .send(Err(e.into()))
                        .map_err(|_| Error::ChannelClosed)?,
                }
            }
            Command::ExistingAlbum(folder, artist_id, reply) => {
                let mut dir = read_dir(&folder).await?;
                let mut albums = Vec::new();
                while let Some(entry) = dir.next_entry().await? {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(folder_name) = path.file_name().and_then(|s| s.to_str()) {
                            // it would be cool to have a parametric system to allow people configure folder names
                            if let Some((year, title)) = folder_name.split_once(" - ") {
                                albums.push((year.to_owned(), title.to_owned()));
                            }
                        }
                    }
                }

                if !albums.is_empty() {
                    let mut query = ReleaseGroupSearchQuery::query_builder();
                    let mut temp = &mut query;
                    let mut first = true;
                    for album in albums {
                        if first {
                            first = false;
                        } else {
                            temp = temp.or();
                        }
                        temp = temp.expr(
                            ReleaseGroupSearchQuery::query_builder()
                                .release_group(&album.1)
                                .and()
                                .first_release_date(&album.0),
                        );
                    }
                    let query = ReleaseGroupSearchQuery::query_builder()
                        .arid(&artist_id)
                        .and()
                        .expr(temp)
                        .build();
                    match ReleaseGroup::search(query).execute().await {
                        Ok(res) => get_queue()
                            .send(Command::MissingAlbum(
                                folder,
                                res.entities
                                    .into_iter()
                                    .map(|release| release.id)
                                    .collect(),
                                artist_id,
                                0,
                                reply,
                            ))
                            .map_err(|_| Error::ChannelClosed)?,
                        Err(e) => reply
                            .send(Err(e.into()))
                            .map_err(|_| Error::ChannelClosed)?,
                    }
                } else {
                    get_queue()
                        .send(Command::MissingAlbum(folder, vec![], artist_id, 0, reply))
                        .map_err(|_| Error::ChannelClosed)?;
                }
            }
            Command::MissingAlbum(folder, albums, artist_id, mut offset, reply) => {
                match ReleaseGroup::browse()
                    .by_artist(&artist_id)
                    .limit(100)
                    .offset(offset)
                    .execute()
                    .await
                {
                    Ok(res) => {
                        offset += res.entities.len() as u16;
                        for release in res.entities {
                            if !albums.contains(&release.id) {
                                println!(
                                    "Missing album {}/{} - {}",
                                    folder.display(),
                                    release.first_release_date.unwrap_or_default().format("%Y"),
                                    release.title,
                                );
                            }
                        }

                        if (offset as i32) < res.count {
                            get_queue()
                                .send(Command::MissingAlbum(
                                    folder, albums, artist_id, offset, reply,
                                ))
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
