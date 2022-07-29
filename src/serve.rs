use crate::client_command::ClientCommand;
use crate::info::WorldInfo;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type WorldInfoReceiver = mpsc::Receiver<WorldInfo>;
type WorldInfoSharedReceiver = Arc<Mutex<WorldInfoReceiver>>;

type ClientCommandSender = mpsc::Sender<ClientCommand>;

pub async fn serve(world_info_rx: WorldInfoReceiver, client_command_tx: ClientCommandSender) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // build our application with some routes
    let app = Router::new()
        .fallback(
            get_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
                .handle_error(|error: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                }),
        )
        // routes are matched from bottom to top, so we have to put the fallback at the
        // top since it matches all routes
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        // .layer(
        //     TraceLayer::new_for_http()
        //         .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        // )
        .layer(Extension(Arc::new(Mutex::new(world_info_rx))))
        .layer(Extension(client_command_tx));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    // tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(world_info_rx): Extension<WorldInfoSharedReceiver>,
    Extension(client_command_tx): Extension<ClientCommandSender>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, world_info_rx, client_command_tx))
}

async fn handle_socket<'a>(
    socket: WebSocket,
    world_info_rx: WorldInfoSharedReceiver,
    client_command_tx: ClientCommandSender,
) {
    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(async move {
        loop {
            if let Some(Ok(Message::Text(msg))) = receiver.next().await {
                if msg == "stop" {
                    client_command_tx.send(ClientCommand::Stop).await.unwrap();
                } else if msg == "start" {
                    client_command_tx.send(ClientCommand::Start).await.unwrap();
                }
            }
        }
    });

    loop {
        if let Some(value) = world_info_rx.lock().await.recv().await {
            // XXX unwrap here, what if this fails?
            let json = serde_json::to_string(&value).unwrap();

            if sender.send(Message::Text(json)).await.is_err() {
                // XXX this isn't the world's best error handling either
                println!("client disconnected");
                return;
            }
        }
    }
}
