use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use poster::auth::AuthClient;

use poster::configuration::get_configuration;
use poster::startup::run;
use poster::telemetry::{get_subscriber, init_subscriber};


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let subscriber = get_subscriber(
        "poster-service".into(),
        "info".into(),
        std::io::stdout
    );
    init_subscriber(subscriber);

    let configuration = get_configuration()
        .expect("Failed to read configuration");

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let auth_client = AuthClient::new(configuration.auth_client.base_url);

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)
        .expect("Failed to bind address");

    run(listener, connection_pool, auth_client)?.await

}