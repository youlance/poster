use actix_extract_multipart::File;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub username: String,
    pub img_url: String,
    pub caption: Option<String>,
    pub likes: i32,
    pub created_at: NaiveDateTime
}

#[derive(Debug, Deserialize, Validate)]
pub struct PostCreate {
    #[validate(length(min = 3, max = 20))]
    pub username: String,
    pub img_file: File,
    #[validate(length(max = 256))]
    pub caption: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostID {
    pub id: Uuid
}

#[derive(Debug, Deserialize, Validate)]
pub struct PostUpdate {
    pub id: Uuid,
    pub caption: Option<String>
}

#[derive(Debug, Deserialize, Validate)]
pub struct FeedFollowing {
    pub page: i32,
    pub followings: Vec<String>
}

#[derive(Debug, Serialize)]
pub struct LatestPosts {
    pub posts: Vec<Post>
}

#[derive(Debug, Serialize)]
pub struct UserPosts {
    pub posts: Vec<Post>
}