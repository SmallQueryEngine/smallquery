use std::str;

#[derive(rust_embed::RustEmbed)]
#[folder = "web_assets/build"]
struct WebAssets;

pub async fn serve_assets(tail: warp::path::Tail) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Requesting asset: {}", tail.as_str());
    if let Some(file) = WebAssets::get(tail.as_str()) {
        let contents = file.data;
        let payload = str::from_utf8(&contents).unwrap().to_owned();
        Ok(Box::new(warp::reply::with_header(
            payload,
            "content-type",
            "text/css", // TODO: Handle other file types
        )))
    } else {
        Err(warp::reject::reject())
    }
}
