use rocket::response::content::{RawCss, RawJavaScript};

#[get("/static/global.css")]
pub async fn global_css() -> RawCss<&'static str> {
    RawCss(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/global.css"
        ))
    )
}

#[get("/static/search.js")]
pub async fn search_js() -> RawJavaScript<&'static str> {
    RawJavaScript(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/search.js"
        ))
    )
}