use rocket::response::content::RawCss;

#[get("/static/global.css")]
pub async fn global_css() -> RawCss<&'static str> {
    RawCss(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/global.css")
        )
    )
}