use std::{collections::BTreeMap, sync::Arc};

use rocket::{State, serde::json::Json};

use crate::{generate::{index::MediaIndex, subtitle::{EpisodeMetadata, MediaMetadata, Sub}}, serve::util::Id};

#[get("/list/<media>/<episode>")]
pub fn list_subtitles<'s>(
    media: Id,
    episode: Id,
    index: &'s State<MediaIndex>
) -> Option<Json<&'s [Sub]>> {
    Some(Json(
        &index.sub_data.get(*media)?
            .get(*episode)?
            .list
    ))
}

#[get("/list/<media>")]
pub fn list_episodes<'s>(
    media: Id,
    index: &'s State<MediaIndex>
) -> Option<Json<BTreeMap<&'s String, &'s Arc<EpisodeMetadata>>>> {
    Some(Json(
        index.sub_data.get(*media)?
            .keys()
            .map(|episode_name| index.episode_data.get_key_value(episode_name).unwrap())
            .collect()
    ))
}

#[get("/list")]
pub fn list_media(
    index: &State<MediaIndex>
) -> Option<Json<&BTreeMap<String, Arc<MediaMetadata>>>> {
    Some(Json(&index.media_data))
}