use rppal::gpio::Gpio;
use std::{net::SocketAddr, sync::Arc, collections::HashMap};

use axum::{
    extract::Query,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

struct AppState {
    gpio: Gpio,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(AppState {
        gpio: Gpio::new().expect("Cannot create GPIO instance."),
    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/led", post(led_control))
        .route("/led", get(led_control_get))
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

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Raspberry Pi LED control"
}

// get pin as output and set state according to request body
async fn led_control(
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
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
            return (StatusCode::SERVICE_UNAVAILABLE, "Failed to get pin");
        }
    }

    (StatusCode::OK, "OK")
}

async fn led_control_get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
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
            return (StatusCode::SERVICE_UNAVAILABLE, "Failed to get pin");
        }
    }

    (StatusCode::OK, "OK")
}
