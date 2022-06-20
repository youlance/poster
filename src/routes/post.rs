use actix_extract_multipart::{File, Multipart};
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{PostDelete, PostCreate, Post, PostUpdate};
use tracing::instrument;


// CRUD: CREATE
#[instrument(
    name = "Creating a new post",
    skip(new_post, pool)
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
    skip(file)
)]
async fn save_file(file: &File, path: &str) -> std::io::Result<()> {

    tokio::fs::write(path, file.data()).await?;

    Ok(())
}

#[instrument(
    name = "Inserting the post to the database",
    skip(pool, new_post)
)]
async fn insert_post(
    pool: &PgPool,
    new_post: &PostCreate,
    img_url: &str
) -> Result<PostDelete, sqlx::Error> {

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
            e
        })?;

    Ok(PostDelete {id})
}



// CRUD: DELETE
#[instrument(
    name = "Deleting the post",
    skip(pool)
)]
pub async fn delete_post(post: web::Json<PostDelete>, pool: web::Data<PgPool>) -> impl Responder {

    let file_name = match del_post(&post, &pool).await {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    if del_file(file_name).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[instrument(
    name = "Removing file from fs"
)]
async fn del_file(file_name: String) -> std::io::Result<()> {
    tokio::fs::remove_file(file_name).await?;
    Ok(())
}

#[instrument(
    name = "Deleting post from database",
    skip(pool)
)]
async fn del_post(post_id: &PostDelete, pool: &PgPool) -> Result<String, sqlx::Error>
{

    let rec = sqlx::query!(
        r#"DELETE FROM posts WHERE id = $1 RETURNING img_url"#,
        post_id.id
    )
        .fetch_one(pool)
        .await?;


    Ok(format!("{}", rec.img_url))
}


#[instrument(
    name = "Fetching post from database",
    skip(pool)
)]
pub async fn get_post(
    post_id: web::Json<PostDelete>,
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
        Err(_) => {
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[instrument(
    name = "Updating post in the database",
    skip(pool)
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