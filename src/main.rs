#[macro_use]
extern crate validator_derive;

mod config;
mod routes;
mod models;

use color_eyre::Result;
use crate::config::Config;
use actix_web::{App, HttpServer, middleware::Logger, web};
use tracing::info;
use crate::routes::app_config;


#[actix_web::main]
async fn main() -> Result<()> {

    let config = Config::from_env()
        .expect("Server configuration");
    
    let pool = config.db_pool().await
        .expect("Database Configuration");
    
    let pool = web::Data::new(pool);

    info!("Starting server at http://{}:{}/", config.host, config.port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(app_config)
            .app_data(pool.clone())
    })
        .bind(format!("{}:{}", config.host, config.port))?
        .run()
        .await?;

    Ok(())
}