#[cfg(not(target_arch = "wasm32"))]
pub use db::DbContext;

pub use plugin::{
    ExistsInDb, External, HandleOrUuid, IsAwarenessBody, SaveFileData, StoragePlugin,
};

#[cfg(not(target_arch = "wasm32"))]
mod db;
mod plugin;
#[cfg(not(target_arch = "wasm32"))]
mod systems;
