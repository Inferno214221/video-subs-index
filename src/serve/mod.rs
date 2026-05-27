// TODO: content-types html, json, gif

#[get("/content/<media>/<episode>/<id>")]
pub fn view_content(media: &str, episode: &str, id: &str) -> String {
    // Maybe just file server for this bit honestly, cause it need to be exclusively cached results
    format!("Accessing {}, {}, {}", media, episode, id)
}

#[put("/content/<media>/<episode>/<id>")]
pub fn create_content(media: &str, episode: &str, id: &str) -> String {
    // TODO: Auth
    format!("Creating {}, {}, {}", media, episode, id)
}

#[get("/search/<media>/<episode>?<q>")]
pub fn search_episode(media: &str, episode: &str, q: &str) -> String {
    format!("Searching {}, {} for {}", media, episode, q)
}

#[get("/search/<media>?<q>")]
pub fn search_media(media: &str, q: &str) -> String {
    format!("Searching {} for {}", media, q)
}

#[get("/search?<q>")]
pub fn search_all(q: &str) -> String {
    todo!("Explicitly deny")
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