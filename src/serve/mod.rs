// TODO: content-types html, json, gif

use std::{collections::BTreeMap, convert::Infallible, num::ParseIntError, path::Path};

use ct_regex::{Regex, regex};
use derive_more::{AsRef, Deref, Display, Error, From};
use hypertext::prelude::*;
use macron_path::path;
use rocket::{Request, State, http::{Status, uri::Origin}, request::{FromParam, FromRequest, Outcome}, response::{Responder, content::RawHtml, status::{BadRequest, Created}}, serde::json::Json, tokio::fs::File};

use crate::{CONTENT_ROOT, video::{MediaIndex, Sub, slice_video}};

// TODO: Catchers for 404, 406, 400, 500

regex!{
    pub AlphaNumeric = r"[\w\-]+"
}

#[derive(Debug, Display, Deref, AsRef)]
#[as_ref(forward)]
pub struct Id<'r>(pub &'r str);

#[derive(Debug, Clone, Display, Error, From)]
pub struct PatternMismatchError;

impl<'r> FromParam<'r> for Id<'r> {
    type Error = PatternMismatchError;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if AlphaNumeric::is_match(param) {
            Ok(Id(param))
        } else {
            Err(PatternMismatchError)
        }
    }
}

regex!{
    pub ArtifactPattern = r"(?<num>\d+)(.gif)?"
}

#[derive(Debug, Clone, Copy, Display, Deref)]
pub struct ArtifactId(pub usize);

#[derive(Debug, Clone, Display, Error, From)]
pub enum ArtifactIdError {
    Pattern,
    Parse(ParseIntError)
}

impl<'r> FromParam<'r> for ArtifactId {
    type Error = ArtifactIdError;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        match ArtifactPattern::do_capture(param) {
            Some(cap) => Ok(ArtifactId(cap.num().parse()?)),
            None => Err(ArtifactIdError::Pattern),
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

#[derive(Debug, Deref, Responder)]
#[response(content_type = "image/gif")]
pub struct Gif(pub File);

impl Gif {
    pub async fn open(path: impl AsRef<Path>) -> Option<Gif> {
        Some(Gif(
            File::open(path).await.ok()?
        ))
    }
}

#[get("/content/<media>/<episode>/<artifact>")]
pub async fn view_content<'r>(
    media: Id<'r>,
    episode: Id<'r>,
    artifact: ArtifactId,
    _gif: &ImplicitGif,
) -> Option<Gif> {
    Gif::open(path!("{CONTENT_ROOT}/{media}/{episode}/{artifact}.gif")).await
}

pub type CreatedSub<'s> = Created<Json<&'s Sub>>;

#[put("/content/<media>/<episode>/<artifact>")]
pub fn create_content<'s>(
    media: Id,
    episode: Id,
    artifact: ArtifactId,
    index: &'s State<MediaIndex>,
    origin: &Origin,
) -> Option<CreatedSub<'s>> {
    let sub = index.get(*media)?
        .get(*episode)?
        .list
        .get(*artifact - 1)?;

    slice_video(
        path!("{CONTENT_ROOT}/{media}/{episode}.mkv"),
        path!("{CONTENT_ROOT}/{media}/{episode}/{artifact}.gif"),
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
pub fn search_all(q: &str) -> BadRequest<&'static str> {
    let _ = q;
    BadRequest("Search without a specified media source is not allowed")
}

#[derive(Debug, Responder, Deref)]
pub struct Html(pub RawHtml<Rendered<String>>);

impl Html {
    pub fn render(from: impl Renderable) -> Html {
        Html(RawHtml(from.render()))
    }
}

#[get("/search")]
pub fn search_page(_html: &ImplicitHtml) -> Html {
    Html::render(
        rsx! {
            <h1>Epic Search Page</h1>
        }
    )
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