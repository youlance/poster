use actix_web::{HttpResponse, Responder, web};
use crate::models::{FeedFollowing, LatestPosts, Post};
use sqlx::PgPool;
use tracing::instrument;

#[instrument(
    name = "Getting latest posts",
    skip(feed, pool)
)]
pub async fn get_latest(feed: web::Json<FeedFollowing>, pool: web::Data<PgPool>) -> impl Responder {

    let query_result = sqlx::query!(
        r#"
        SELECT * FROM posts
        WHERE username = ANY($1)
        ORDER BY created_at
        LIMIT 10 OFFSET $2
        "#,
        &feed.followings[..],
        feed.page as i64 * 10
    )
        .fetch_all(pool.as_ref())
        .await;

    let records = match query_result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to execute query {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    };

    let posts: Vec<Post> = records.into_iter()
        .map(|r| {
            Post {
                id: r.id,
                username: r.username,
                img_url: r.img_url,
                caption: r.caption,
                likes: r.likes,
                created_at: r.created_at
            }
        }).collect();


    let latest = LatestPosts {
        posts
    };

    HttpResponse::Ok()
        .json(latest)
}
