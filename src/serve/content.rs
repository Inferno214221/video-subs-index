use std::num::ParseIntError;

use ct_regex::{Regex, regex};
use derive_more::{Deref, Display, Error, From};
use rocket::{State, http::uri::Origin, request::FromParam , response::status::Created, serde::json::Json};

use crate::{generate::{index::MediaIndex, subtitle::Sub, video}, serve::{files, util::{Gif, Id, ImplicitGif}}};

regex! {
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

#[get("/content/<media>/<episode>/<artifact>")]
pub async fn view_content<'r>(
    media: Id<'r>,
    episode: Id<'r>,
    artifact: ArtifactId,
    _gif: &ImplicitGif,
) -> Option<Gif> {
    Gif::open(files::artifact(media, episode, *artifact)).await
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
    let sub = index.episode_data.get(*media)?
        .get(*episode)?.1
        .list
        .get(*artifact - 1)?;

    video::slice_content(media, episode, *artifact, sub);

    Some(
        Created::new(origin.path().as_str().to_owned())
            .body(Json(sub))
    )
}
