use reqwest::{Client, StatusCode};

use std::future::{ready, Ready};

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, web};
use actix_web::cookie::Cookie;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized};

use futures_util::future::LocalBoxFuture;


pub struct AuthClient {
    pub http_client: Client,
    pub base_url: String,
}

impl AuthClient {
    pub fn new(base_url: String) -> Self {
        AuthClient {
            http_client: Client::new(),
            base_url
        }
    }

    pub async fn authorize<'a>(&self, username_cookie: Option<Cookie<'a>>, access_token_cookie: Option<Cookie<'a>>) -> Result<bool, Error> {
        let username_cookie = username_cookie.ok_or(ErrorBadRequest("username cookie not available"))?;
        let access_token_cookie = access_token_cookie.ok_or(ErrorBadRequest("access_token cookie not available"))?;

        let body = Verify {
            username: username_cookie.value(),
            access_token: access_token_cookie.value()
        };

        let res = self.http_client
            .post(&self.base_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Unable to access auth service: {:?}", e);
                ErrorInternalServerError("Authentication service not available")
            })?;
        match res.status() {
            StatusCode::OK => Ok(true),
            _ => Ok(false)
        }
    }
}


#[derive(serde::Serialize)]
pub struct Verify<'a> {
    username: &'a str,
    access_token: &'a str
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middle-ware's call method gets called with normal request.
pub struct Author;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Author
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorMiddleware { service }))
    }
}

pub struct AuthorMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthorMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // println!("Hi from start. You requested: {}", req.path());

        // println!("{:?}", cookies);
        let username = req.cookie("username");
        let token = req.cookie("access_token");

        let client = req.app_data::<web::Data<AuthClient>>()
            .expect("AuthClient not found in server data domain").clone();


        let fut = self.service.call(req);


        Box::pin(async move {

            let auth_fut = client.authorize(username, token);
            let authorized = auth_fut.await?;
            if !authorized {
                return Err(ErrorUnauthorized("User is not authorized"))
            }

            let res = fut.await?;

            Ok(res)
        })


    }
}