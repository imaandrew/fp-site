use axum::{body::StreamBody, extract::Path, response::IntoResponse, routing::get, Router};
use path_absolutize::Absolutize;
use std::path::Path as SysPath;
use tokio_util::io::ReaderStream;

mod builder;
use builder::build_patches;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/get-patch/:tag/:version", get(get_patch));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_patch(Path((tag, version)): Path<(String, String)>) -> impl IntoResponse {
    let path = format!("patches/{}/{}.xdelta", tag, version);
    let path = SysPath::new(&path).absolutize().unwrap();
    let path_str = path.to_str().unwrap();

    if !path_str.starts_with(&format!(
        "{}/patches/",
        std::env::current_dir().unwrap().to_str().unwrap()
    )) {
        panic!();
    }

    if !path_str.ends_with(".xdelta") {
        panic!();
    }

    if path_str
        .replace(std::env::current_dir().unwrap().to_str().unwrap(), "")
        .matches('/')
        .count()
        != 3
    {
        panic!();
    }

    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(_) => {
            build_patches(tag, version);
            tokio::fs::File::open(&path).await.unwrap()
        }
    };

    let stream = ReaderStream::new(file);
    StreamBody::new(stream)
}
