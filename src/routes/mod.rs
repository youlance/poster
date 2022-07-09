mod post;
mod feed;
mod files;

use actix_cors::Cors;

use actix_web::{HttpResponse, web};
use actix_web::web::ServiceConfig;
use crate::auth::Author;
use crate::routes::feed::get_latest;
use crate::routes::post::{delete_post, get_post, upload_post, update_post, get_use_posts};
use actix_files as fs;

pub fn app_config(config: &mut ServiceConfig) {

    let health_resource = web::resource("/")
        .route(web::get().to(health));

    let posts_resource = web::scope("/posts")
//        .wrap(Author)
        .route("", web::post().to(upload_post))
        .route("", web::delete().to(delete_post))
        .route("", web::get().to(get_post))
        .route("/{username}", web::get().to(get_use_posts))
        .route("", web::patch().to(update_post));

//    let static_files = web::scope("/files")
//        .route("/{filename}", web::get().to(files));

    let feed_resource = web::resource("/latest")
//        .wrap(Author)
        .route(web::post().to(get_latest));

    config.service(health_resource);
    config.service(posts_resource);
    config.service(fs::Files::new("/files","./files").show_files_listing());
    config.service(feed_resource);
}


pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}
