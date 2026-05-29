use rocket::fs::{FileServer, relative};
use video_subs_index::{serve::*, generate::index::MediaIndex};

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(MediaIndex::build_from_fs())
        .mount(
            "/",
            routes![
                view_content,
                create_content,
                search_episode,
                search_media,
                search_all,
                search_page_episode,
                search_page_media,
                search_page,
                list_subtitles,
                list_episodes,
                list_media
            ]
        )
        .mount("/static", FileServer::from(relative!("res")))
}