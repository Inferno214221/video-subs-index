// TODO: content-types html, json, gif

use std::{collections::BTreeMap, convert::Infallible, path::PathBuf};

use ct_regex::{Regex, regex};
use derive_more::{AsRef, Deref, Display};
use rocket::{Request, State, fs::{NamedFile, relative}, http::Status, request::{FromParam, FromRequest, Outcome}, serde::json::Json};
use srt_subtitles_parser::Subtitle;

use crate::video::MediaIndex;

// TODO: Catchers for 404, 406, 400, 500

regex!{
    pub AlphaNumeric = r"[\w\-]+"
}

pub const CONTENT_ROOT: &str = relative!("content");

#[derive(Debug, Display, Deref, AsRef)]
#[as_ref(forward)]
pub struct Id<'r> {
    pub name: &'r str,
}

impl<'r> FromParam<'r> for Id<'r> {
    type Error = ();

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if AlphaNumeric::is_match(param) {
            Ok(Id {
                name: param,
            })
        } else {
            Err(())
        }
    }
}

pub struct GifContentType;


#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r GifContentType {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Infallible> {
        match request.content_type() {
            Some(content_type) if content_type.is_gif() => Outcome::Success(&GifContentType),
            None => Outcome::Success(&GifContentType),
            _ => Outcome::Forward(Status::NotAcceptable),
        }
    }
}

#[get("/content/<media>/<episode>/<id>")]
pub async fn view_content<'r>(
    media: Id<'r>,
    episode: Id<'r>,
    id: Id<'r>,
    _gif: &'r GifContentType
) -> Option<NamedFile> {
    NamedFile::open(
        [CONTENT_ROOT, &media, &episode, &id]
            .iter()
            .collect::<PathBuf>()
    ).await.ok()
}

#[put("/content/<media>/<episode>/<id>")]
pub fn create_content(media: &str, episode: &str, id: &str) -> (Status, String) {
    // TODO: Auth
    (Status::Ok, format!("Creating {}, {}, {}", media, episode, id))
}

#[get("/search/<media>/<episode>?<q>")]
pub fn search_episode<'r>(
    media: Id<'r>,
    episode: Id<'r>,
    q: &'r str,
    index: &'r State<MediaIndex>
) -> Option<Json<Vec<Subtitle>>> {
    Some(Json(
        index.get(*media)?
            .get(*episode)?
            .search_with_query(q).collect()
    ))
}

#[get("/search/<media>?<q>")]
pub fn search_media<'r>(
    media: Id<'r>,
    q: &'r str,
    index: &'r State<MediaIndex>
) -> Option<Json<BTreeMap<String, Vec<Subtitle>>>> {
    Some(Json(
        index.get(*media)?.iter()
            .map(|(episode, index)| (
                episode.clone(),
                index.search_with_query(q).collect()
            ))
            .collect()
    ))
}

#[get("/search?<q>")]
#[allow(unused_variables)]
pub fn search_all(q: &str) -> (Status, &'static str) {
    (Status::BadRequest, "Search without a specified media source is not allowed")
}

#[get("/search")]
pub fn search_page() -> String {
    "Search page".into()
}

#[get("/list/<media>/<episode>")]
pub fn list_subtitles(media: &str, episode: &str) -> String {
    format!("List of subtitles for {}, {}", media, episode)
}

#[get("/list/<media>")]
pub fn list_episodes(media: &str) -> String {
    format!("List of episodes for {}", media)
}

#[get("/list")]
pub fn list_media() -> String {
    "List of media".into()
}