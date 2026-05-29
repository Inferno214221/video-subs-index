use std::collections::BTreeMap ;

use hypertext::{Buffer, context::Node, prelude::*};
use rocket::{State, http::{Accept, MediaType, Status}, response::status::BadRequest, serde::json::Json};

use crate::{generate::{index::MediaIndex, subtitle::Sub}, serve::util::{Html, Id, ImplicitHtml}};

#[derive(Debug, Responder)]
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
    let results = index.get(*media)?
        .get(*episode)?
        .index
        .search_with_query(query);

    Some(match accept.unwrap_or(&Accept::JSON).preferred().media_type() {
        ty if ty == &MediaType::JSON || ty == &MediaType::Any => {
            HtmlOrJson::Json(Json(results))
        },
        ty if ty == &MediaType::HTML => {
            HtmlOrJson::Html(Html::render(
                SearchPageWithResults {
                    form: SearchPageForm {
                        index,
                        media: Some(&media),
                        episode: Some(&episode),
                    },
                    results: SubDisplay {
                        media: &media,
                        episode: &episode,
                        subs: &results,
                    },
                }
            ))
        },
        _ => {
            HtmlOrJson::Neither(Status::NotAcceptable)
        },
    })
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
                media: Some(*media),
                episode: Some(*episode),
            }
        }
    )
}

#[get("/search/<media>?<query>")]
pub fn search_media<'s>(
    media: Id,
    query: &str,
    index: &'s State<MediaIndex>,
    accept: Option<&Accept>,
) -> Option<HtmlOrJson<BTreeMap<&'s str, Vec<Sub>>>> {
    let results = index.get(*media)?
        .iter()
        .map(|(episode, data)| (
            episode.as_str(),
            data.index.search_with_query(query)
        ))
        .collect::<BTreeMap<_, _>>();

    Some(match accept.unwrap_or(&Accept::JSON).preferred().media_type() {
        ty if ty == &MediaType::JSON || ty == &MediaType::Any => {
            HtmlOrJson::Json(Json(results))
        },
        ty if ty == &MediaType::HTML => {
            HtmlOrJson::Html(Html::render(
                SearchPageWithResults {
                    form: SearchPageForm {
                        index,
                        media: Some(&media),
                        episode: None,
                    },
                    results: SubDisplayWithEpisode {
                        media: &media,
                        subs: results
                    },
                }
            ))
        },
        _ => {
            HtmlOrJson::Neither(Status::NotAcceptable)
        },
    })
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
                media: Some(*media),
                episode: None,
            }
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
            }
        }
    )
}

struct PageWrapper<R: Renderable, S: AsRef<str>> {
    title: S,
    children: R,
}

impl<R: Renderable, S: AsRef<str>> Renderable for PageWrapper<R, S> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            head {
                title {
                    "Quotes - " (self.title.as_ref())
                }
                link rel="stylesheet" href="/static/global.css";
            }

            body {
                nav {
                    h1 { "Quotes" }
                }

                main {
                    (self.children)
                }
            }
        }
        .render_to(buffer);
    }
}

struct SelectedOption<'v, R: Renderable> {
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

struct SearchPageForm<'i> {
    index: &'i MediaIndex,
    media: Option<&'i str>,
    episode: Option<&'i str>,
}

impl<'i> Renderable for SearchPageForm<'i> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            form #search-form action="/search" onsubmit="return onSubmit();" {
                label for="search-media" { "Media: " }
                select #search-media onchange="onChangeMedia()" {
                    @if let Some(selected_media) = self.media {
                        option value="" { "(none)" }
                        @for media in self.index.keys() {
                            SelectedOption value=media selected=(media == selected_media) {
                                (media)
                            }
                        }
                    } @else {
                        option value="" selected { "(none)" }
                        @for media in self.index.keys() {
                            option value=media { (media) }
                        }
                    }
                }
                br;

                label for="search-episode" { "Episode: " }
                select #search-episode onchange="onChangeEpisode()" {
                    option value="" { "(any)" }
                    @if let Some(selected_media) = self.media &&
                        let Some(episodes) = self.index.get(selected_media)
                    {
                        @if let Some(selected_episode) = self.episode {
                            @for episode in episodes.keys() {
                                SelectedOption value=episode selected=(episode == selected_episode) {
                                    (episode)
                                }
                            }
                        } @else {
                            @for episode in episodes.keys() {
                                option value=episode { (episode) }
                            }
                        }
                    }
                }
                br;

                input #search-bar type="search" placeholder="Quote" name="query";
                input #search-button type="submit" value="Search";

                script src="/static/search.js" {}
            }
        }
        .render_to(buffer);
    }
}

struct SearchPage<'i> {
    form: SearchPageForm<'i>,
}

impl<'i> Renderable for SearchPage<'i> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            PageWrapper title="Search" {
                h2 {
                    "Search Quotes"
                }

                (self.form)
            }
        }
        .render_to(buffer);
    }
}

struct SearchPageWithResults<'i, R: Renderable> {
    form: SearchPageForm<'i>,
    results: R,
}

impl<'i, R: Renderable> Renderable for SearchPageWithResults<'i, R> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            PageWrapper title="Search" {
                h2 {
                    "Search Results"
                }

                (self.form)

                (self.results)

                script src="/static/search.js" {}
            }
        }
        .render_to(buffer);
    }
}

pub struct SubDisplay<'d> {
    pub media: &'d str,
    pub episode: &'d str,
    pub subs: &'d Vec<Sub>,
}

impl<'s> Renderable for SubDisplay<'s> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            ul {
                @for subtitle in self.subs {
                    li {
                        a href=(format!("/content/{}/{}/{}", self.media, self.episode, subtitle.index)) {
                            (subtitle.index)
                        }
                        ": "
                        (subtitle.text)
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
}

impl<'s> Renderable for SubDisplayWithEpisode<'s> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            ul {
                @for episode in self.subs.keys() {
                    li { (episode) }
                    SubDisplay
                        media=(self.media)
                        episode=(episode)
                        subs=(self.subs.get(episode).unwrap());
                }
            }
        }
        .render_to(buffer);
    }
}