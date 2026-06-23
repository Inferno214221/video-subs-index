use std::{fmt::Display, num::ParseIntError};

use ct_regex::{AnonRegex, regex};
use derive_more::{Display, Error, From};
use rocket::{State, http::uri::Origin, request::FromParam , response::status::Created, serde::json::Json};

use crate::{generate::{index::MediaIndex, subtitle::Sub, video::{self, VidRange}}, serve::{files, util::{Gif, Id, ImplicitGif}}};

// #[derive(Debug, Clone, Copy, Display, Deref)]
// pub struct ArtifactId(pub usize);

#[derive(Debug, Clone, Display, Error, From)]
pub enum ArtifactIdError {
    Pattern,
    Parse(ParseIntError)
}

// impl<'r> FromParam<'r> for ArtifactId {
//     type Error = ArtifactIdError;

//     fn from_param(param: &'r str) -> Result<Self, Self::Error> {
//         match regex!(r"(?<num>\d+)(.gif)?").do_capture(param) {
//             Some(cap) => Ok(ArtifactId(cap.num().parse()?)),
//             None => Err(ArtifactIdError::Pattern),
//         }
//     }
// }

#[derive(Debug, Clone, Copy)]
pub struct ArtifactRange {
    pub start: usize,
    pub end: usize,
}

impl Display for ArtifactRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

impl ArtifactRange {
    pub fn single(num: usize) -> ArtifactRange {
        ArtifactRange { start: num, end: num }
    }
}

impl<'r> FromParam<'r> for ArtifactRange {
    type Error = ArtifactIdError;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        match regex!(r"(?<start>\d+)(-(?<end>\d+))?(.gif)?").do_capture(param) {
            Some(cap) => Ok(ArtifactRange {
                start: cap.start().parse()?,
                end: cap.end().unwrap_or(cap.start()).parse()?,
            }),
            None => Err(ArtifactIdError::Pattern),
        }
    }
}

#[get("/content/<media>/<episode>/<artifact>")]
pub async fn view_content<'r>(
    media: Id<'r>,
    episode: Id<'r>,
    artifact: ArtifactRange,
    _gif: &ImplicitGif,
) -> Option<Gif> {
    Gif::open(files::artifact(media, episode, artifact)).await
}

pub type CreatedSub<'s> = Created<Json<[&'s Sub; 2]>>;

#[put("/content/<media>/<episode>/<artifact>")]
pub fn create_content<'s>(
    media: Id,
    episode: Id,
    artifact: ArtifactRange,
    index: &'s State<MediaIndex>,
    origin: &Origin,
) -> Option<CreatedSub<'s>> {
    let sub_list = &index.episode_data.get(*media)?
        .get(*episode)?
        .subs
        .list;

    let first = sub_list.get(artifact.start - 1)?;
    let last = sub_list.get(artifact.end - 1)?;

    let range = VidRange::new(first, last).expect("subtitle exceeds maximum duration");

    video::slice_content(media, episode, artifact, range);

    Some(
        Created::new(origin.path().as_str().to_owned())
            .body(Json([first, last]))
    )
}
