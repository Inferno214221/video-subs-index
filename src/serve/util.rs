use std::{convert::Infallible, path::Path};

use ct_regex::{Regex, regex};
use derive_more::{AsRef, Deref, Display, Error, From};
use hypertext::{Renderable, Rendered};
use rocket::{Request, http::Status, request::{FromParam, FromRequest, Outcome}, response::content::RawHtml, tokio::fs::File};

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

#[derive(Debug, Responder, Deref)]
pub struct Html(pub RawHtml<Rendered<String>>);

impl Html {
    pub fn render(from: impl Renderable) -> Html {
        Html(RawHtml(from.render()))
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