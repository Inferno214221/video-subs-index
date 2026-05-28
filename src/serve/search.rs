use std::collections::BTreeMap ;

use hypertext::prelude::*;
use rocket::{State, response::status::BadRequest, serde::json::Json};

use crate::{serve::util::{Html, Id, ImplicitHtml}, video::{MediaIndex, Sub}};

// TODO: content-types html, json, gif

#[get("/search/<media>/<episode>?<q>")]
pub fn search_episode(
    media: Id,
    episode: Id,
    q: &str,
    index: &State<MediaIndex>
) -> Option<Json<Vec<Sub>>> {
    Some(Json(
        index.get(*media)?
            .get(*episode)?
            .index
            .search_with_query(q)
    ))
}

#[get("/search/<media>?<q>")]
pub fn search_media<'s>(
    media: Id,
    q: &str,
    index: &'s State<MediaIndex>
) -> Option<Json<BTreeMap<&'s str, Vec<Sub>>>> {
    Some(Json(
        index.get(*media)?
            .iter()
            .map(|(episode, data)| (
                episode.as_str(),
                data.index.search_with_query(q)
            ))
            .collect()
    ))
}

#[get("/search?<q>")]
pub fn search_all(q: &str) -> BadRequest<&'static str> {
    let _ = q;
    BadRequest("Search without a specified media source is not allowed")
}

#[get("/search")]
pub fn search_page(_html: &ImplicitHtml) -> Html {
    Html::render(
        rsx! {
            <h1>Epic Search Page</h1>
        }
    )
}
