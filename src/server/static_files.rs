use axum::{
    body::Body,
    http::{header, Response, StatusCode, Uri},
    response::IntoResponse,
};
use rust_embed::RustEmbed;
use std::borrow::Cow;

/// Embedded dashboard assets
#[derive(RustEmbed)]
#[folder = "web/dist"]
struct DashboardAssets;

/// Create a service that serves embedded static files
pub fn serve_static() -> axum::routing::MethodRouter {
    axum::routing::get(static_handler)
}

/// Handle static file requests
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try to get the file directly
    if let Some(content) = get_embedded_file(path) {
        return content;
    }

    // For SPA routing: if no file found and not an API route, serve index.html
    if !path.starts_with("api/") {
        if let Some(content) = get_embedded_file("index.html") {
            return content;
        }
    }

    // Return 404
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}

/// Get embedded file content with proper content type
fn get_embedded_file(path: &str) -> Option<Response<Body>> {
    let path = if path.is_empty() { "index.html" } else { path };

    DashboardAssets::get(path).map(|content| {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let body: Cow<'static, [u8]> = content.data;

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, "public, max-age=3600")
            .body(Body::from(body.into_owned()))
            .unwrap()
    })
}
