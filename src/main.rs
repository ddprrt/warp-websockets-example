use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use futures::{FutureExt, StreamExt};

use tokio::sync::{
    mpsc::{self},
    RwLock,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

static NEXT_USERID: AtomicUsize = AtomicUsize::new(1);

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

#[tokio::main]
async fn main() {
    let users = Users::default();
    let users = warp::any().map(move || users.clone());

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let chat = warp::path("ws")
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| ws.on_upgrade(move |socket| user_connected(socket, users)));

    let files = warp::fs::dir("./static");

    let routes = chat.or(hello).or(files);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn user_connected(ws: WebSocket, users: Users) {
    let my_id = NEXT_USERID.fetch_add(1, Ordering::Relaxed);
    let (user_tx, mut user_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(rx.forward(user_tx).map(|res| {
        if let Err(e) = res {
            println!("socket send error {}", e);
        }
    }));
    users.write().await.insert(my_id, tx);

    let users_disc = users.clone();

    while let Some(result) = user_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                Message::text(format!("Error {}", e.to_string()))
            }
        };
        user_message(msg, &users).await;
    }

    user_disconnected(my_id, &users_disc).await;

}

async fn user_message(msg: Message, users: &Users) {
    if let Ok(_s_msg) = msg.to_str() {
        for (&_uid, tx) in users.read().await.iter() {
            tx.send(Ok(msg.clone())).unwrap();
        }
    }
}


async fn user_disconnected(my_id: usize, users: &Users) {
    println!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}
