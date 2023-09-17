use axum::extract::State;
use axum::routing::get;
use axum::{routing::post, Json, Router};
use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Condvar};
use std::time::Duration;
use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::messages::RunnerMessage;
use steadyum_api_types::objects::{RegionList, WarmBodyObjectSet, WatchedObjects};
use steadyum_api_types::partitionner::{
    AssignRunnerRequest, AssignRunnerResponse, InsertObjectsRequest, RunnerInitializedRequest,
    StartStopRequest, ASSIGN_RUNNER_ENDPOINT, INSERT_OBJECTS_ENDPOINT, LIST_REGIONS_ENDPOINT,
    PARTITIONNER_PORT, RUNNER_INITIALIZED_ENDPOINT, START_STOP_ENDPOINT,
};
use steadyum_api_types::rapier::parry::bounding_volume::BoundingVolume;
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::{runner_zenoh_commands_key, ZenohContext};
use tokio::sync::Mutex;
use uuid::Uuid;
use zenoh::prelude::r#async::AsyncResolve;

const MAX_PENDING_RUNNERS: u32 = 10;

pub struct Runner {
    pub process: Child,
    pub uuid: Uuid,
    pub region: SimulationBounds,
    pub port: u32,
    pub started: bool,
}

pub struct LiveRunners {
    pub next_port_id: u32,
    pub assigned: HashMap<SimulationBounds, Runner>,
    pub pending: Vec<Runner>,
    pub uninitialized: HashMap<Uuid, Runner>,
}

impl Default for LiveRunners {
    fn default() -> Self {
        Self {
            next_port_id: 10_000,
            assigned: HashMap::default(),
            pending: vec![],
            uninitialized: HashMap::default(),
        }
    }
}

struct SharedState {
    runners: Mutex<LiveRunners>,
    num_pending: AtomicU32,
    resume_runner_allocation: Condvar,
    assign_runner_lock: Mutex<()>,
    zenoh: ZenohContext,
    kvs: Mutex<KvsContext>,
    running: AtomicBool,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            runners: Mutex::new(LiveRunners::default()),
            num_pending: AtomicU32::new(0),
            resume_runner_allocation: Condvar::new(),
            assign_runner_lock: Mutex::new(()),
            zenoh: ZenohContext::new().unwrap(),
            kvs: Mutex::new(KvsContext::new().unwrap()),
            running: AtomicBool::new(false),
        }
    }
}

#[derive(Clone)]
struct AppState {
    data: Arc<SharedState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            data: Arc::new(SharedState::default()),
        }
    }
}

#[tokio::main]
async fn main() {
    init_log();
    let state = AppState::default();
    let state_clone = state.clone();

    std::thread::spawn(move || {
        smol::block_on(runner_allocator_loop(state_clone));
    });

    let app = Router::new()
        .route(ASSIGN_RUNNER_ENDPOINT, post(assign_runner))
        .route(RUNNER_INITIALIZED_ENDPOINT, post(runner_initialized))
        .route(INSERT_OBJECTS_ENDPOINT, post(insert_objects))
        .route(LIST_REGIONS_ENDPOINT, get(list_regions))
        .route(START_STOP_ENDPOINT, post(start_stop))
        .with_state(state);
    axum::Server::bind(&format!("0.0.0.0:{}", PARTITIONNER_PORT).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn start_stop(State(state): State<AppState>, Json(payload): Json<StartStopRequest>) {
    state.data.running.store(payload.running, Ordering::SeqCst);
    let runners: Vec<_> = state
        .data
        .runners
        .lock()
        .await
        .assigned
        .values()
        .map(|runner| runner.uuid)
        .collect();

    for uuid in runners {
        put_runner_message(
            &state.data.zenoh,
            uuid,
            RunnerMessage::StartStop {
                running: payload.running,
            },
        )
        .await
        .unwrap();
    }
}

async fn list_regions(State(state): State<AppState>) -> Json<RegionList> {
    let runners = state.data.runners.lock().await;
    Json(RegionList {
        keys: runners
            .assigned
            .values()
            .map(|r| r.region.runner_key())
            .collect(),
        ports: runners.assigned.values().map(|r| r.port).collect(),
    })
}

async fn insert_objects(State(state): State<AppState>, Json(payload): Json<InsertObjectsRequest>) {
    log::info!("Inserting {} objects.", payload.bodies.len());

    let mut region_to_objects = HashMap::new();

    // Group object by region.
    for body in payload.bodies {
        // TODO: not calculating islands here will induce a 1-step delay between the
        //       time the objects are simulated and the time they become aware of any
        //       potential island merge across the boundary.
        let aabb = body.cold.shape.compute_aabb(&body.warm.position);

        if body.cold.body_type.is_dynamic() {
            let region = SimulationBounds::from_aabb(&aabb, SimulationBounds::DEFAULT_WIDTH);
            region_to_objects
                .entry(region)
                .or_insert_with(Vec::new)
                .push(body);
        } else {
            for region in SimulationBounds::intersecting_aabb(
                aabb.loosened(1.0),
                SimulationBounds::DEFAULT_WIDTH,
            ) {
                region_to_objects
                    .entry(region)
                    .or_insert_with(Vec::new)
                    .push(body.clone());
            }
        }
    }

    // Send objects to runners.
    for (region, bodies) in region_to_objects {
        let runner =
            assign_runner(State(state.clone()), Json(AssignRunnerRequest { region })).await;

        log::info!("Inserting {} objects to {}", bodies.len(), runner.uuid);

        // Send message to the runner.
        let message = RunnerMessage::AssignIsland {
            bodies,
            impulse_joints: vec![],
        };
        put_runner_message(&state.data.zenoh, runner.uuid, message)
            .await
            .unwrap();
    }
}

async fn runner_initialized(
    State(state): State<AppState>,
    Json(payload): Json<RunnerInitializedRequest>,
) {
    let mut runners = state.data.runners.lock().await;

    if let Some(runner) = runners.uninitialized.remove(&payload.uuid) {
        log::info!("Runner {:?} acked initialization.", payload.uuid);
        runners.pending.push(runner);
    }
}

async fn assign_runner(
    State(state): State<AppState>,
    Json(payload): Json<AssignRunnerRequest>,
) -> Json<AssignRunnerResponse> {
    // TODO: this basically makes this endpoint operate completely sequentially.
    //       How could we avoid this?
    let _lock_guard = state.data.assign_runner_lock.lock();

    let runners = state.data.runners.lock().await;
    if let Some(runner) = runners.assigned.get(&payload.region) {
        return Json(AssignRunnerResponse {
            region: payload.region,
            uuid: runner.uuid,
        });
    }

    log::info!(
        "No region assigned to region {:?} yet. Num pending: {}",
        payload.region,
        runners.pending.len()
    );
    drop(runners);

    // No runner exist for this region yet, assign one.
    loop {
        let mut runners = state.data.runners.lock().await;
        if let Some(mut runner) = runners.pending.pop() {
            let uuid = runner.uuid;
            runner.region = payload.region;
            runners.assigned.insert(payload.region, runner);
            let time_origin = 1; // TODO

            // Reset warm and watch sets.
            // TODO: this should either be done elsewhere to not block the
            //       partitionner, or we should add a "scene UUID" to these objects
            //       so viewer can’t mix them up.
            state
                .data
                .kvs
                .lock()
                .await
                .put_warm(
                    &payload.region.runner_key(),
                    &WarmBodyObjectSet {
                        timestamp: time_origin,
                        objects: vec![],
                    },
                )
                .unwrap();

            // Send message to the runner.
            let message = RunnerMessage::AssignRegion {
                region: payload.region,
                time_origin,
            };
            put_runner_message(&state.data.zenoh, uuid, message)
                .await
                .unwrap();

            drop(runners);

            state.data.num_pending.fetch_sub(1, Ordering::SeqCst);
            state.data.resume_runner_allocation.notify_all();

            return Json(AssignRunnerResponse {
                region: payload.region,
                uuid,
            });
        } else {
            // Free the lock before waiting.
            drop(runners);
            // There is no runner available yet. Wait for at least one to come online.
            std::thread::sleep(Duration::from_millis(5))
        }
    }
}

fn init_log() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.init();
}

async fn runner_allocator_loop(state: AppState) {
    let condvar_mutex = std::sync::Mutex::new(());
    loop {
        while state.data.num_pending.load(Ordering::Relaxed) < MAX_PENDING_RUNNERS {
            // NOTE: we lock before spawning the process to be sure we have a chance to
            //       add the runner to the pending queue before the runner acknowledges
            //       its initialization.
            let uuid = Uuid::new_v4();
            log::info!("Spawning new runner: {:?}.", uuid);
            let mut locked_runners = state.data.runners.lock().await;
            let process = Command::new("steadyum-runner.exe")
                .args(["--uuid".to_string(), format!("{}", uuid.to_u128_le())])
                .spawn()
                .unwrap();
            let runner = Runner {
                process,
                uuid,
                region: SimulationBounds::smallest(),
                port: 0, // TODO: might become needed in the future?
                started: false,
            };
            locked_runners.uninitialized.insert(runner.uuid, runner);
            drop(locked_runners);

            state.data.num_pending.fetch_add(1, Ordering::SeqCst);
        }

        let condvar_lock = condvar_mutex.lock().unwrap();
        let _ = state.data.resume_runner_allocation.wait(condvar_lock);
    }
}

pub async fn put_runner_message(
    zenoh: &ZenohContext,
    uuid: Uuid,
    message: RunnerMessage,
) -> anyhow::Result<()> {
    let message_str = serde_json::to_string(&message)?;
    let zenoh_key = runner_zenoh_commands_key(uuid);
    let publisher = zenoh
        .session
        .declare_publisher(&zenoh_key)
        .res()
        .await
        .unwrap();
    publisher.put(message_str).res().await.unwrap();
    Ok(())
}
