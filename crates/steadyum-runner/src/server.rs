use crate::runner::SharedSimulationState;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use rapier::math::Real;
use rapier::pipeline::QueryFilter;
use std::net::SocketAddr;
use steadyum_api_types::queries::{RayCastQuery, RayCastResponse};

pub async fn raycast(
    State(state): State<SharedSimulationState>,
    Json(query): Json<RayCastQuery>,
) -> Json<RayCastResponse> {
    let mut sim_state_guard = state.0.write().unwrap();
    let mut sim_state = &mut *sim_state_guard;

    sim_state
        .query_pipeline
        .update(&sim_state.bodies, &sim_state.colliders);
    let response = sim_state
        .query_pipeline
        .cast_ray(
            &sim_state.bodies,
            &sim_state.colliders,
            &query.ray,
            Real::MAX,
            true,
            QueryFilter::default(),
        )
        .map(|(h, toi)| {
            let hit = sim_state
                .colliders
                .get(h)
                .and_then(|co| co.parent())
                .and_then(|h| sim_state.body2uuid.get(&h));
            RayCastResponse {
                hit: hit.copied(),
                toi,
            }
        })
        .unwrap_or_default();

    Json(response)
}

pub async fn serve(port: u32, state: SharedSimulationState) {
    let app = Router::new()
        .route("/raycast", get(raycast))
        .with_state(state);

    let addr: SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .expect("Unable to parse socket address");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

pub fn spawn_server(port: u32, state: SharedSimulationState) {
    std::thread::spawn(move || {
        use tokio::runtime;
        let rt = runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();
        rt.block_on(serve(port, state));
    });
}
