// TODO: content-types html, json, gif

use std::{collections::BTreeMap, convert::Infallible};

use ct_regex::{Regex, regex};
use derive_more::{AsRef, Deref, Display};
use macron_path::path;
use rocket::{Request, State, fs::NamedFile, http::{Status, uri::Origin}, request::{FromParam, FromRequest, Outcome}, response::status::{BadRequest, Created}, serde::json::Json};

use crate::{CONTENT_ROOT, video::{MediaIndex, Sub, slice_video}};

// TODO: Catchers for 404, 406, 400, 500

regex!{
    pub AlphaNumeric = r"[\w\-]+"
}

#[derive(Debug, Display, Deref, AsRef)]
#[as_ref(forward)]
pub struct Id<'r>(pub &'r str);

impl<'r> FromParam<'r> for Id<'r> {
    type Error = ();

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if AlphaNumeric::is_match(param) {
            Ok(Id(param))
        } else {
            Err(())
        }
    }
}

pub struct ImplicitGif;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r ImplicitGif {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Infallible> {
        match request.content_type() {
            Some(content_type) if content_type.is_gif() => Outcome::Success(&ImplicitGif),
            None => Outcome::Success(&ImplicitGif),
            _ => Outcome::Forward(Status::NotAcceptable),
        }
    }
}

pub struct ImplicitHtml;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r ImplicitHtml {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Infallible> {
        match request.content_type() {
            Some(content_type) if content_type.is_gif() => Outcome::Success(&ImplicitHtml),
            None => Outcome::Success(&ImplicitHtml),
            _ => Outcome::Forward(Status::NotAcceptable),
        }
    }
}

#[get("/content/<media>/<episode>/<artifact>")]
pub async fn view_content<'r>(
    media: Id<'r>,
    episode: Id<'r>,
    artifact: usize,
    _gif: &ImplicitGif,
) -> Option<NamedFile> {
    NamedFile::open(
        path!("{CONTENT_ROOT}/{media}/{episode}/{artifact}.gif")
    ).await.ok()
}

#[put("/content/<media>/<episode>/<artifact>")]
pub fn create_content<'s>(
    media: Id,
    episode: Id,
    artifact: usize,
    index: &'s State<MediaIndex>,
    origin: &Origin,
) -> Option<Created<Json<&'s Sub>>> {
    let target = path!("{CONTENT_ROOT}/{media}/{episode}/{artifact}.gif");

    let sub = index.get(*media)?
        .get(*episode)?
        .list
        .get(artifact - 1)?;

    slice_video(
        path!("{CONTENT_ROOT}/{media}/{episode}.mkv"),
        target,
        sub
    );

    Some(
        Created::new(origin.path().as_str().to_owned())
            .body(Json(sub))
    )
}

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
#[allow(unused_variables)]
pub fn search_all(q: &str) -> BadRequest<&'static str> {
    BadRequest("Search without a specified media source is not allowed")
}

#[get("/search")]
pub fn search_page(_html: &ImplicitHtml) -> String {
    "Search page".into()
}

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