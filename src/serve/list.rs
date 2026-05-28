use rocket::{State, serde::json::Json};

use crate::{serve::util::Id, video::{MediaIndex, Sub}};

#[get("/list/<media>/<episode>")]
pub fn list_subtitles<'s>(
    media: Id,
    episode: Id,
    index: &'s State<MediaIndex>
) -> Option<Json<&'s [Sub]>> {
    Some(Json(
        &index.get(*media)?
            .get(*episode)?
            .list
    ))
}

#[get("/list/<media>")]
pub fn list_episodes<'s>(media: Id, index: &'s State<MediaIndex>) -> Option<Json<Vec<&'s str>>> {
    Some(Json(
        index.get(*media)?
            .keys()
            .map(String::as_str)
            .collect()
    ))
}

#[get("/list")]
pub fn list_media(index: &State<MediaIndex>) -> Option<Json<Vec<&str>>> {
    Some(Json(
        index.keys()
            .map(String::as_str)
            .collect()
    ))
}