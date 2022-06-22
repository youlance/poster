use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::auth::AuthClient;
use crate::routes::*;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    auth_client: AuthClient
) -> Result<Server, std::io::Error> {
    // Wrap hte connection in a smart pointer
    let db_pool = web::Data::new(db_pool);
    let auth_client = web::Data::new(auth_client);

    let server = HttpServer::new(move || {

        App::new()
            .wrap(TracingLogger::default())
            .configure(app_config)
            .app_data(db_pool.clone())
            .app_data(auth_client.clone())
    })
        .listen(listener)?
        .run();

    Ok(server)
}