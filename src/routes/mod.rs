mod post;
mod feed;

use actix_web::{HttpResponse, web};
use actix_web::web::ServiceConfig;
use crate::auth::Author;
use crate::routes::feed::get_latest;
use crate::routes::post::{delete_post, get_post, upload_post, update_post};

pub fn app_config(config: &mut ServiceConfig) {

    let health_resource = web::resource("/")
        .route(web::get().to(health));

    let posts_resource = web::resource("/posts")
        .wrap(Author)
        .route(web::post().to(upload_post))
        .route(web::delete().to(delete_post))
        .route(web::get().to(get_post))
        .route(web::patch().to(update_post));

    let feed_resource = web::resource("/latest")
        .wrap(Author)
        .route(web::post().to(get_latest));

    config.service(health_resource);
    config.service(posts_resource);
    config.service(feed_resource);
}


pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}