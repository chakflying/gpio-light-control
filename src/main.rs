use rppal::gpio::Gpio;
use std::{collections::HashMap, fs, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};

struct AppState {
    gpio: Gpio,
    index: String,
    icon: String,
    manifest: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(AppState {
        gpio: Gpio::new().expect("Cannot create GPIO instance."),
        index: fs::read_to_string("src/index.html")
            .unwrap_or(String::from("Failed to read index.html")),
        icon: fs::read_to_string("src/icon.svg").unwrap_or(String::from("Failed to read icon.svg")),
        manifest: fs::read_to_string("src/manifest.json")
            .unwrap_or(String::from("Failed to read manifest.json")),
    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/led", post(led_control))
        .route("/led", get(led_control_get))
        .route("/:file", get(file))
        .route("/", get(root))
        .with_state(shared_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 8234));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    return Html(state.index.clone());
}

// basic handler that responds with a static string
async fn file(State(state): State<Arc<AppState>>, Path(file): Path<String>) -> Response {
    return match file.as_str() {
        "manifest.json" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            state.manifest.clone(),
        ).into_response(),
        "icon.svg" => (
            StatusCode::OK, 
            [(header::CONTENT_TYPE, "image/svg+xml")],
            state.icon.clone(),
        ).into_response(),
        _ => return (StatusCode::NOT_FOUND, "").into_response(),
    }
}

// get pin as output and set state according to request body
async fn led_control(State(state): State<Arc<AppState>>, body: String) -> Response {
    let pin = state.gpio.get(23);
    match pin {
        Ok(pin) => {
            let mut light_pin = pin.into_output();
            light_pin.set_reset_on_drop(false);

            if body.trim() == "on" {
                light_pin.set_high();
            } else {
                light_pin.set_low();
            }
        }
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Failed to get pin").into_response();
        }
    }

    Redirect::to("/").into_response()
}

async fn led_control_get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let pin = state.gpio.get(23);
    match pin {
        Ok(pin) => {
            let mut light_pin = pin.into_output();
            light_pin.set_reset_on_drop(false);

            if params.contains_key("on") {
                light_pin.set_high();
            } else {
                light_pin.set_low();
            }
        }
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Failed to get pin").into_response();
        }
    }

    Redirect::to("/").into_response()
}
