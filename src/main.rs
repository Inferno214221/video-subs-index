use video_subs_index::serve::*;

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![
        view_content,
        create_content,
        search_episode,
        search_media,
        search_all,
        search_page,
        list_subtitles,
        list_episodes,
        list_media
    ])
}