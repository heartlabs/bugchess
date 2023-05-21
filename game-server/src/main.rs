use chrono::Utc;
use std::{fs, fs::File, io::Write};

use warp::{reject, Filter};

#[derive(Debug)]
struct NoString;

impl reject::Reject for NoString {}

const ERROR_REPORTS_DIR: &str = "error_reports";
const MAX_ERROR_REPORTS: usize = 50;

#[tokio::main]
async fn main() {
    let error_report = warp::post()
        .and(warp::path("error_report"))
        // Only accept bodies smaller than 1mb...
        .and(warp::body::content_length_limit(1024 * 1024))
        .and(warp::body::bytes())
        .and_then(|bytes: bytes::Bytes| async {
            if let Ok(body_string) = String::from_utf8(bytes.into()) {
                println!("Received error report: \n{}", body_string);
                std::fs::create_dir_all(ERROR_REPORTS_DIR)
                    .and_then(|()| fs::read_dir(ERROR_REPORTS_DIR))
                    .map(|r| {
                        if r.count() > MAX_ERROR_REPORTS {
                            panic!("Too many reports")
                        }
                    })
                    .map(|()| format!("{}/{}.json", ERROR_REPORTS_DIR, Utc::now()))
                    .and_then(File::create)
                    .and_then(|mut file| file.write(body_string.clone().into_bytes().as_slice()))
                    .map(|_| body_string)
                    .map_err(|_| reject::custom(NoString {}))
            } else {
                println!("Could not receive error report");
                Err(reject::custom(NoString {}))
            }
        })
        .map(|string: String| string)
        .with(warp::cors::cors().allow_any_origin());

    println!("Started game server.");

    warp::serve(error_report).run(([0, 0, 0, 0], 3030)).await
}
