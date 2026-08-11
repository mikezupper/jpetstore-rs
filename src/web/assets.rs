use axum::extract::Path;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::web::AppError;

// The pet pictures, embedded in the binary at compile time. Fifteen files
// don't justify a static-file crate or a directory that has to ship next
// to the executable — an explicit match keeps the single-binary story true
// and the whole mechanism readable. Adding a file means adding a line,
// and the compiler yells if the path is wrong.
pub async fn image(Path(name): Path<String>) -> Response {
    macro_rules! gif {
        ($file:literal) => {
            (
                [(header::CONTENT_TYPE, "image/gif")],
                include_bytes!(concat!("../../static/images/", $file)).as_slice(),
            )
                .into_response()
        };
    }

    match name.as_str() {
        "fish1.gif" => gif!("fish1.gif"),
        "fish2.gif" => gif!("fish2.gif"),
        "fish3.gif" => gif!("fish3.gif"),
        "fish4.gif" => gif!("fish4.gif"),
        "dog1.gif" => gif!("dog1.gif"),
        "dog2.gif" => gif!("dog2.gif"),
        "dog4.gif" => gif!("dog4.gif"),
        "dog5.gif" => gif!("dog5.gif"),
        "dog6.gif" => gif!("dog6.gif"),
        "snake1.gif" => gif!("snake1.gif"),
        "lizard1.gif" => gif!("lizard1.gif"),
        "cat1.gif" => gif!("cat1.gif"),
        "cat2.gif" => gif!("cat2.gif"),
        "bird1.gif" => gif!("bird1.gif"),
        "banner_fish.gif" => gif!("banner_fish.gif"),
        "banner_cats.gif" => gif!("banner_cats.gif"),
        "banner_dogs.gif" => gif!("banner_dogs.gif"),
        "banner_reptiles.gif" => gif!("banner_reptiles.gif"),
        "banner_birds.gif" => gif!("banner_birds.gif"),
        "bird2.gif" => gif!("bird2.gif"),
        _ => AppError::NotFound.into_response(),
    }
}
