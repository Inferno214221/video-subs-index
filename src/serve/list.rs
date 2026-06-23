use std::{collections::BTreeMap, sync::Arc};

use rocket::{State, serde::json::Json};

use crate::{generate::{index::MediaIndex, subtitle::{Episode, Media, Sub}}, serve::util::Id};

#[get("/list/<media>/<episode>")]
pub fn list_subtitles<'s>(
    media: Id,
    episode: Id,
    index: &'s State<MediaIndex>
) -> Option<Json<&'s [Sub]>> {
    Some(Json(
        &index.episode_data.get(*media)?
            .get(*episode)?
            .subs
            .list
    ))
}

#[get("/list/<media>")]
pub fn list_episodes<'s>(
    media: Id,
    index: &'s State<MediaIndex>
) -> Option<Json<BTreeMap<&'s str, &'s Arc<Episode>>>> {
    Some(Json(
        index.get_episodes(*media)?
    ))
}

#[get("/list")]
pub fn list_media(
    index: &State<MediaIndex>
) -> Option<Json<&BTreeMap<Box<str>, Arc<Media>>>> {
    Some(Json(&index.media_data))
}