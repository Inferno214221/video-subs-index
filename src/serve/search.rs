use std::collections::BTreeMap ;

use hypertext::{Buffer, Raw, context::Node, prelude::*};
use rocket::{State, response::status::BadRequest, serde::json::Json};

use crate::{serve::util::{Html, Id, ImplicitHtml}, video::{MediaIndex, Sub}};

// TODO: content-types html, json, gif

#[get("/search/<media>/<episode>?<query>")]
pub fn search_episode(
    media: Id,
    episode: Id,
    query: &str,
    index: &State<MediaIndex>
) -> Option<Json<Vec<Sub>>> {
    Some(Json(
        index.get(*media)?
            .get(*episode)?
            .index
            .search_with_query(query)
    ))
}

#[get("/search/<media>/<episode>")]
pub fn search_page_episode(
    media: Id,
    episode: Id,
    index: &State<MediaIndex>
) -> Html {
    Html::render(
        SearchPage {
            index,
            media: Some(*media),
            episode: Some(*episode),
        }
    )
}

#[get("/search/<media>?<query>")]
pub fn search_media<'s>(
    media: Id,
    query: &str,
    index: &'s State<MediaIndex>
) -> Option<Json<BTreeMap<&'s str, Vec<Sub>>>> {
    Some(Json(
        index.get(*media)?
            .iter()
            .map(|(episode, data)| (
                episode.as_str(),
                data.index.search_with_query(query)
            ))
            .collect()
    ))
}

#[get("/search/<media>")]
pub fn search_page_media(
    media: Id,
    index: &State<MediaIndex>
) -> Html {
    Html::render(
        SearchPage {
            index,
            media: Some(*media),
            episode: None,
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
            index,
            media: None,
            episode: None,
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

struct SearchPage<'i> {
    index: &'i MediaIndex,
    media: Option<&'i str>,
    episode: Option<&'i str>,
}

impl<'i> Renderable for SearchPage<'i> {
    fn render_to(&self, buffer: &mut Buffer<Node>) {
        maud! {
            PageWrapper title="Search" {
                h2 {
                    "Search Quotes"
                }

                form #search-form action="/search" onsubmit="return onSubmit();" {
                    label for="search-media" { "Media: " }
                    select #search-media onchange="onChangeMedia()" {
                        @if let Some(selected_media) = self.media {
                            option value="" { "(none)" }
                            @for media in self.index.keys() {
                                @if media == selected_media {
                                    option value=(media) selected { (media) }
                                } @else {
                                    option value=(media) { (media) }
                                }
                            }
                        } @else {
                            option value="" selected { "(none)" }
                            @for media in self.index.keys() {
                                option value=(media) { (media) }
                            }
                        }
                    }
                    br;
                    label for="search-episode" { "Episode: " }
                    select #search-episode onchange="onChangeEpisode()" {
                        option value="" selected { "(any)" }
                        @if let Some(selected_media) = self.media &&
                            let Some(episodes) = self.index.get(selected_media)
                        {
                            @if let Some(selected_episode) = self.episode {
                                @for media in episodes.keys() {
                                    @if media == selected_episode {
                                        option value=(media) selected { (media) }
                                    } @else {
                                        option value=(media) { (media) }
                                    }
                                }
                            } @else {
                                @for media in episodes.keys() {
                                    option value=(media) { (media) }
                                }
                            }
                        }
                    }
                    br;
                    input #search-bar type="search" placeholder="Quote" name="query";
                    input #search-button type="submit" value="Search";
                }

                script {
                    (Raw::dangerously_create(include_str!("./search.js")))
                }
            }
        }
        .render_to(buffer);
    }
}