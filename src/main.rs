use axum::{
    extract::Multipart,
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use std::collections::HashMap;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use tokio::fs::File;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
// Specify your desired upload folder here
const UPLOAD_DIR: &str = "uploads";

// handler to upload image or file
async fn upload_image(
    mut files: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    // Ensure the upload directory exists or create it
    tokio::fs::create_dir_all(UPLOAD_DIR).await.unwrap();

    while let Some(file) = files.next_field().await.unwrap() {
        // this is the name which is sent in formdata from frontend or whoever called the api, i am
        // using it as category, we can get the filename from file data
        let category = file.name().unwrap().to_string();
        // name of the file with extention
        let name = file.file_name().unwrap().to_string();
        // file data
        let data = file.bytes().await.unwrap();
        // the path of file to store on aws s3 with file name and extention
        // timestamp_category_filename => 14-12-2022_01:01:01_customer_somecustomer.jpg
        let key = format!(
            "{}_{}_{}",
            chrono::Utc::now().format("%d%m%Y%H%M%S"),
            &category,
            &name
        );

        println!("key `{}`", key);

        let file_path = PathBuf::from(UPLOAD_DIR).join(key);

        match File::create(file_path).await {
            Ok(mut file) => {
                file.write_all(&data).await.unwrap();
            },
            Err(e) => return Ok(Json(serde_json::json!({
                                 "status": "error",
                                 "message": "Error creating file"
                            }))),
        }

    }
    // send the urls in response
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "File(s) uploaded successfully!"
    })))
}


#[tokio::main]
async fn main() {
    // configuration logging and initiate it
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Path to the directory you want to serve files from
    let static_files_path = UPLOAD_DIR; // Change this to your public directory path

    // Setup the router
    let app = Router::new()

        // route for testing if api is running correctly
        .route("/", get(|| async move { "File Upload API" }))

        //route for uploading image or any file
        .route("/upload", post(upload_image))

        // set your cors config
        .nest_service("/static", ServeDir::new(static_files_path));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .expect("failed to start server");
}
