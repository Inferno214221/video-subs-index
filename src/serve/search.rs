use std::{collections::BTreeMap, convert::identity, fs} ;

use derive_more::{Display, From};
use hypertext::{Buffer, context::Node, prelude::*};
use rocket::{State, http::{Accept, MediaType, Status}, response::status::BadRequest, serde::json::Json};
use srt_subtitles_parser::Timestamp;

use crate::{generate::{index::MediaIndex, subtitle::Sub}, serve::{files, util::{Html, Id, ImplicitHtml}}};

pub trait IntoHtmlOrJson: Sized {
    type JsonInner: Sized;

    fn html(self) -> Html;

    fn json_inner(self) -> Self::JsonInner;

    fn json(self) -> Json<Self::JsonInner> {
        Json(self.json_inner())
    }

    fn into_html_or_json(self, accept: Option<&Accept>) -> HtmlOrJson<Self::JsonInner> {
        for media_type in accept.unwrap_or(&Accept::JSON).media_types() {
            match media_type {
                ty if ty == &MediaType::JSON
                    || ty == &MediaType::Any => return self.json().into(),
                ty if ty == &MediaType::HTML => return self.html().into(),
                _ => (),
            }
        }
        HtmlOrJson::Neither(Status::NotAcceptable)
    }
}

#[derive(Debug, Responder, From)]
pub enum HtmlOrJson<T> {
    Html(Html),
    Json(Json<T>),
    Neither(Status),
}

#[get("/search/<media>/<episode>?<query>")]
pub fn search_episode(
    media: Id,
    episode: Id,
    query: &str,
    index: &State<MediaIndex>,
    accept: Option<&Accept>,
) -> Option<HtmlOrJson<Vec<Sub>>> {
    let data = index.get_subs(*media, *episode);

    let mut results = if query.is_empty() {
        data.list.clone().into_vec()
    } else {
        data.index.search_with_query(query)
    };
    results.sort_by_key(|sub| sub.index);

    Some(
        SearchEpisode {
            index,
            media: &media,
            episode: &episode,
            results,
        }.into_html_or_json(accept)
    )
}

pub struct SearchEpisode<'r> {
    index: &'r MediaIndex,
    media: &'r str,
    episode: &'r str,
    results: Vec<Sub>,
}

impl<'r> IntoHtmlOrJson for SearchEpisode<'r> {
    type JsonInner = Vec<Sub>;

    fn html(self) -> Html {
        Html::render(
            SearchPage {
                form: SearchPageForm {
                    index: self.index,
                    media: Some(self.media),
                    episode: Some(self.episode),
                },
                results: SubDisplay {
                    media: self.media,
                    episode: self.episode,
                    subs: &self.results,
                    index: self.index,
                },
            }
        )
    }

    fn json_inner(self) -> Self::JsonInner {
        self.results
    }
}

#[get("/search/<media>/<episode>")]
pub fn search_page_episode(
    media: Id,
    episode: Id,
    index: &State<MediaIndex>
) -> Html {
    Html::render(
        SearchPage {
            form: SearchPageForm {
                index,
                media: Some(&media),
                episode: Some(&episode),
            },
            results: ()
        }
    )
}

#[get("/search/<media>?<query>")]
pub fn search_media<'r>(
    media: Id<'r>,
    query: &'r str,
    index: &'r State<MediaIndex>,
    accept: Option<&'r Accept>,
) -> Option<HtmlOrJson<BTreeMap<&'r str, Vec<Sub>>>> {
    let results = index.episode_data.get(*media)?
        .iter()
        .map(|(episode, data)| {
            let mut subs = data.subs.index.search_with_query(query);
            subs.sort_by_key(|sub| sub.index);
            (&**episode, subs)
        })
        .collect::<BTreeMap<_, _>>();

    Some(
        SearchMedia {
            index,
            media: &media,
            results,
        }.into_html_or_json(accept)
    )
}

pub struct SearchMedia<'r> {
    index: &'r MediaIndex,
    media: &'r str,
    results: BTreeMap<&'r str, Vec<Sub>>,
}

impl<'r> IntoHtmlOrJson for SearchMedia<'r> {
    type JsonInner = BTreeMap<&'r str, Vec<Sub>>;

    fn html(self) -> Html {
        Html::render(
            SearchPage {
                form: SearchPageForm {
                    index: self.index,
                    media: Some(self.media),
                    episode: None,
                },
                results: SubDisplayWithEpisode {
                    media: self.media,
                    subs: self.results,
                    index: self.index,
                },
            }
        )
    }

    fn json_inner(self) -> Self::JsonInner {
        self.results
    }
}

#[get("/search/<media>")]
pub fn search_page_media(
    media: Id,
    index: &State<MediaIndex>
) -> Html {
    Html::render(
        SearchPage {
            form: SearchPageForm {
                index,
                media: Some(&media),
                episode: None,
            },
            results: ()
        }
    )
}

#[get("/search?<query>")]
pub fn search_all(query: &str) -> BadRequest<&'static str> {
    let _ = query;
    BadRequest("Search without a specified media source is not allowed")
}

#[get("/search")]
pub fn search_page(index: &State<MediaIndex>, _html: &ImplicitHtml) -> Html {
    Html::render(
        SearchPage {
            form: SearchPageForm {
                index,
                media: None,
                episode: None,
            },
            results: ()
        }
    )
}

pub struct PageWrapper<R: Renderable, S: AsRef<str>> {
    title: S,
    children: R,
}

impl<R: Renderable, S: AsRef<str>> Renderable for PageWrapper<R, S> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            head {
                title {
                    "Subtitle Index - " (self.title.as_ref())
                }
                link rel="stylesheet" href="/static/global.css";
            }

            body {
                nav {
                    h1 { "Subtitle Index" }
                }

                main {
                    (self.children)
                }
            }
        }
        .render_to(buffer);
    }
}

pub struct SelectedOption<'v, R: Renderable> {
    value: &'v str,
    selected: bool,
    children: R,
}

impl<'v, R: Renderable> Renderable for SelectedOption<'v, R> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            @if self.selected {
                option value=(self.value) selected { (self.children) }
            } @else {
                option value=(self.value) { (self.children) }
            }
        }
        .render_to(buffer);
    }
}

pub struct SearchPageForm<'i> {
    index: &'i MediaIndex,
    media: Option<&'i str>,
    episode: Option<&'i str>,
}

impl<'i> Renderable for SearchPageForm<'i> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            form #search-form action="/search" onsubmit="return onSubmit();" {
                select #search-media onchange="onChangeMedia()" {
                    @if let Some(selected_media) = self.media {
                        option value="" { "(No Media Source)" }
                        @for (media_name, media) in self.index.media_data.iter() {
                            SelectedOption value=media_name selected=(&**media_name == selected_media) {
                                %(media)
                            }
                        }
                    } @else {
                        option value="" selected { "(No Media Source)" }
                        @for (media_name, media) in self.index.media_data.iter() {
                            option value=media_name {
                                %(media)
                            }
                        }
                    }
                }

                select #search-episode onchange="onChangeEpisode()" {
                    option value="" { "(Any Episode)" }
                    @if let Some(selected_media) = self.media &&
                        let Some(episodes) = self.index.get_episodes(selected_media)
                    {
                        @if let Some(selected_episode) = self.episode {
                            @for (episode_name, episode) in episodes.iter() {
                                SelectedOption
                                    value=episode_name
                                    selected=(episode_name == &selected_episode)
                                {
                                    %(episode)
                                }
                            }
                        } @else {
                            @for (episode_name, episode) in episodes.iter() {
                                option value=episode_name {
                                    %(episode)
                                }
                            }
                        }
                    }
                }

                input #search-bar type="search" placeholder="Seach subtitle contents" name="query";
                input #search-button type="submit" value="Search";

                script src="/static/search.js" {}
            }
        }
        .render_to(buffer);
    }
}

pub struct SearchPage<'i, R: Renderable> {
    form: SearchPageForm<'i>,
    results: R,
}

impl<'i, R: Renderable> Renderable for SearchPage<'i, R> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            PageWrapper title="Search" {
                h2 { "Subtitle Search" }
                (self.form)
                (self.results)
            }
        }
        .render_to(buffer);
    }
}

pub struct SubDisplay<'d> {
    pub media: &'d str,
    pub episode: &'d str,
    pub subs: &'d Vec<Sub>,
    pub index: &'d MediaIndex,
}

impl<'s> Renderable for SubDisplay<'s> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            div .sub-list {
                @for subtitle in self.subs {
                    @let link = format!(
                        "/content/{}/{}/{}",
                        self.media,
                        self.episode,
                        subtitle.index
                    );
                    div .sub-display {
                        @let quote = format!("\"{}\"", subtitle.text);
                        @if fs::exists(
                            files::artifact(self.media, self.episode, subtitle.index as usize)
                        ).is_ok_and(identity) {
                            a href=link { img src=link alt=quote; }
                        } @else {
                            div .sub-quote {
                                span { (quote) }
                                button .sub-generate
                                    onclick=(format!(
                                        "generate('{}', '{}', '{}')",
                                        self.media,
                                        self.episode,
                                        subtitle.index
                                    ))
                                    { "Generate" }
                            }
                        }
                        div .sub-time {
                            " (" %(DispTime(&subtitle.start)) " - " %(DispTime(&subtitle.end)) ")"
                        }
                    }
                }
            }
        }
        .render_to(buffer);
    }
}

pub struct SubDisplayWithEpisode<'d> {
    pub media: &'d str,
    pub subs: BTreeMap<&'d str, Vec<Sub>>,
    pub index: &'d MediaIndex,
}

impl<'s> Renderable for SubDisplayWithEpisode<'s> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            div {
                @for episode in self.subs.keys() {
                    @if let Some(subs) = self.subs.get(episode) && !subs.is_empty() {
                        h3 { %(self.index.get_episode(self.media, episode)) }
                        SubDisplay
                            media=(self.media)
                            episode=episode
                            subs=subs
                            index=(self.index);
                    }

                }
            }
        }
        .render_to(buffer);
    }
}

#[derive(Display)]
#[display("{:02}:{:02}:{:02}", _0.hours, _0.minutes, _0.seconds)]
pub struct DispTime<'a>(pub &'a Timestamp);