use axum::{
    extract::{
        ws::WebSocketUpgrade,
        State,
    },
    http::Method,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod ws;

use config::Config;
use ws::gateway::Gateway;

/// Shared application state
#[derive(Clone)]
struct AppState {
    pool: sqlx::PgPool,
    config: Config,
    gateway: Gateway,
}

// Implement FromRef for each piece of state so Axum extractors work
impl axum::extract::FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl axum::extract::FromRef<AppState> for Gateway {
    fn from_ref(state: &AppState) -> Self {
        state.gateway.clone()
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zanverse_server=debug,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env();
    let listen_addr = config.listen_addr.clone();

    // Database pool
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");

    // Create uploads directory
    tokio::fs::create_dir_all(&config.upload_dir)
        .await
        .expect("Failed to create upload dir");

    let gateway = Gateway::new();

    let state = AppState {
        pool,
        config: config.clone(),
        gateway,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = Router::new()
        // Auth
        .route("/api/auth/register", post(routes::auth::register))
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/refresh", post(routes::auth::refresh))
        .route("/api/auth/logout", post(routes::auth::logout))
        // Users
        .route("/api/users/@me", get(routes::users::get_me).patch(routes::users::update_me))
        .route("/api/users/@me/dms", get(routes::users::list_dms).post(routes::users::create_dm))
        .route("/api/users/{user_id}", get(routes::users::get_user))
        // Servers
        .route(
            "/api/servers",
            get(routes::servers::list_servers).post(routes::servers::create_server),
        )
        .route(
            "/api/servers/{server_id}",
            get(routes::servers::get_server)
                .patch(routes::servers::update_server)
                .delete(routes::servers::delete_server),
        )
        .route(
            "/api/servers/{server_id}/members",
            get(routes::servers::list_members),
        )
        .route(
            "/api/servers/{server_id}/members/{user_id}",
            delete(routes::servers::remove_member),
        )
        .route(
            "/api/servers/{server_id}/invites",
            get(routes::servers::list_invites).post(routes::servers::create_invite),
        )
        .route(
            "/api/servers/{server_id}/invites/{code}",
            delete(routes::servers::delete_invite),
        )
        .route("/api/invites/{code}/join", post(routes::servers::join_via_invite))
        // Channels
        .route(
            "/api/servers/{server_id}/channels",
            get(routes::channels::list_channels).post(routes::channels::create_channel),
        )
        .route(
            "/api/channels/{channel_id}",
            patch(routes::channels::update_channel).delete(routes::channels::delete_channel),
        )
        // Messages
        .route(
            "/api/channels/{channel_id}/messages",
            get(routes::messages::list_messages).post(routes::messages::send_message),
        )
        .route(
            "/api/channels/{channel_id}/messages/{message_id}",
            patch(routes::messages::edit_message).delete(routes::messages::delete_message),
        )
        // DMs
        .route(
            "/api/dms/{dm_channel_id}/messages",
            get(routes::users::list_dm_messages).post(routes::users::send_dm_message),
        )
        // WebSocket
        .route("/ws", get(ws_handler))
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::Extension(config))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .expect("Failed to bind");

    tracing::info!("Zanverse Chat server listening on {listen_addr}");
    axum::serve(listener, app).await.expect("Server error");
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let jwt_secret = state.config.jwt_secret.clone();
    let gateway = state.gateway.clone();
    ws.on_upgrade(move |socket| async move {
        gateway.handle_connection(socket, jwt_secret).await;
    })
}
