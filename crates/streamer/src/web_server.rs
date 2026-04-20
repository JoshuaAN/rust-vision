use axum::{
    Router,
    body::Body,
    extract::Path,
    http::{Response, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../dashboard/dist/"]
struct Assets;

pub async fn start_web_server(port: u16) {
    let app = Router::new()
        .route("/*path", get(static_handler))
        .fallback(get(index_handler));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("Web Dashboard available at http://localhost:{}", port);
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> impl IntoResponse {
    static_handler(Path("index.html".to_string())).await
}

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap(),
    }
}
