use std::str::FromStr;
use actix_extract_multipart::{File, Multipart};
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{PostID, PostCreate, Post, PostUpdate, UserPosts};
use tracing::instrument;


// CRUD: CREATE
#[instrument(
    name = "Creating a new post",
    skip(new_post, pool)
    fields(
        username = %new_post.username
    )
)]
pub async fn upload_post(new_post: Multipart<PostCreate>, pool: web::Data<PgPool>) -> impl Responder {
    let file_extension = match new_post.img_file.file_type().as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        _ => return HttpResponse::UnsupportedMediaType().finish()
    };

    let file_path = format!("./files/{}.{}", Uuid::new_v4(), file_extension);

    if save_file(&new_post.img_file, &file_path).await.is_err() {
        return HttpResponse::InternalServerError().finish()
    }

    match insert_post(&pool, &new_post, &file_path).await {
        Ok(id) => HttpResponse::Ok().body(
            serde_json::to_string(&id).unwrap()
        ),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[instrument(
    name = "Saving the file in fs",
    skip(file, path),
    fields(
    file_path = %path
    )
)]
async fn save_file(file: &File, path: &str) -> std::io::Result<()> {

    tokio::fs::write(path, file.data()).await
        .map_err(|e| {
            tracing::error!("Unable to save file {} : {:?}", path, e);
            e
        })?;

    Ok(())
}

#[instrument(
    name = "Inserting the post to the database",
    skip(pool, new_post, img_url)
)]
async fn insert_post(
    pool: &PgPool,
    new_post: &PostCreate,
    img_url: &str
) -> Result<PostID, sqlx::Error> {

    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO posts (id, username, img_url, caption, likes, created_at)
        VALUES ($1, $2, $3, $4, DEFAULT, DEFAULT)
        "#,
        id,
        &new_post.username,
        &img_url,
        new_post.caption.as_ref(),
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query {:?}", e);
            e
        })?;

    Ok(PostID {id})
}



// CRUD: DELETE
#[instrument(
    name = "Deleting the post",
    skip(pool),
    fields(
        post_id = %post.id
    )
)]
pub async fn delete_post(post: web::Json<PostID>, pool: web::Data<PgPool>) -> impl Responder {

    let file_name = match del_post(&post, &pool).await {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    if del_file(file_name).await.is_err() {
        // don't do anything
        // return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[instrument(
    name = "Removing file from fs",
    skip(file_name),
    fields(
        file_path = %file_name
    )
)]
async fn del_file(file_name: String) -> std::io::Result<()> {
    tokio::fs::remove_file(&file_name).await
        .map_err(|e| {
            tracing::error!("Unable to delete file {}: {:?}", file_name, e);
            e
        })?;
    Ok(())
}

#[instrument(
    name = "Deleting post from database",
    skip(pool),
    fields(
        post_id = %post_id.id
    )
)]
async fn del_post(post_id: &PostID, pool: &PgPool) -> Result<String, sqlx::Error>
{

    let rec = sqlx::query!(
        r#"DELETE FROM posts WHERE id = $1 RETURNING img_url"#,
        post_id.id
    )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query {:?}", e);
            e
        })?;

    Ok(format!("{}", rec.img_url))
}


#[instrument(
    name = "Fetching post from database",
    skip(pool, post_id),
    fields(
    post_id = %post_id.id
    )
)]
pub async fn get_post(
    post_id: web::Json<PostID>,
    pool: web::Data<PgPool>
) -> impl Responder {

    let post = sqlx::query_as!(
        Post,
        r#"SELECT * FROM posts WHERE id = $1"#,
        post_id.id
    )
        .fetch_one(pool.as_ref())
        .await;

    match post {
        Ok(p) => {
            HttpResponse::Ok().body(
                serde_json::to_string(&p).unwrap()
            )
        },
        Err(e) => {
            tracing::error!("Failed to execute query {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_single_post(
    path: web::Path<(String,)>,
    pool: web::Data<PgPool>
) -> impl Responder {
    let id_str = path.into_inner().0;
    let id: Uuid = match Uuid::from_str(id_str.as_str()) {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to parse path {} to uuid: {:?}", id_str, e);
            return HttpResponse::BadRequest().finish();
        }
    };

    let post = sqlx::query_as!(
        Post,
        r#"SELECT * FROM posts WHERE id = $1"#,
        id
    )
        .fetch_one(pool.as_ref())
        .await;

    match post {
        Ok(p) => {
            HttpResponse::Ok().body(
                serde_json::to_string(&p).unwrap()
            )
        },
        Err(e) => {
            tracing::error!("Failed to execute query {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }

}

pub async fn get_use_posts(
    path: web::Path<(String,)>,
    pool: web::Data<PgPool>
) -> impl Responder {

    let username = path.into_inner().0;

    let query_result = sqlx::query!(
        r#"
        SELECT * FROM posts
        WHERE username = $1
        ORDER BY created_at
        "#,
        username
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

    let user_posts = UserPosts {
        posts
    };

    HttpResponse::Ok()
        .json(user_posts)

}

#[instrument(
    name = "Updating post in the database",
    skip(pool, update),
    fields(
        post_id = %update.id
    )
)]
pub async fn update_post(
    update: web::Json<PostUpdate>,
    pool: web::Data<PgPool>
) -> impl Responder {

    match sqlx::query!(
        r#"
        UPDATE posts
        SET caption = $1
        WHERE id = $2
        "#,
        update.caption,
        update.id
    )
        .execute(pool.as_ref())
        .await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}