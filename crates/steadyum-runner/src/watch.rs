use crate::runner::RunnerCommand;
use flume::Sender;
use std::collections::HashSet;
use std::time::Duration;
use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::objects::WatchedObjects;
use steadyum_api_types::simulation::SimulationBounds;

pub fn spawn_watch_loop(bounds: SimulationBounds, commands_snd: Sender<RunnerCommand>) {
    let mut kvs = KvsContext::new().unwrap();

    std::thread::spawn(move || {
        let nbh_regions = bounds.neighbors_to_watch();
        let nbh_watch_keys = nbh_regions.map(|nbh| nbh.watch_kvs_key());

        loop {
            let mut watched = vec![];
            for (nbh, nbh_key) in nbh_regions.iter().zip(nbh_watch_keys.iter()) {
                if let Ok(data) = kvs.get_with_str_key::<WatchedObjects>(nbh_key) {
                    watched.push((data, *nbh));
                }
            }

            // if !watched.objects.is_empty() {
            //     dbg!("Found watched: {}", watched.objects.len());
            // }
            commands_snd.send(RunnerCommand::SetWatchedSet(watched));
            // std::thread::sleep(Duration::from_secs_f32(1.0)); // TODO
        }
    });
}

pub fn read_watched_objects(
    kvs: &mut KvsContext,
    bounds: SimulationBounds,
) -> Vec<(WatchedObjects, SimulationBounds)> {
    let nbh_regions = bounds.neighbors_to_watch();
    let nbh_watch_keys = nbh_regions.map(|nbh| nbh.watch_kvs_key());

    let mut watched = vec![];
    for (nbh, nbh_key) in nbh_regions.iter().zip(nbh_watch_keys.iter()) {
        if let Ok(data) = kvs.get_with_str_key::<WatchedObjects>(nbh_key) {
            watched.push((data, *nbh));
        }
    }

    watched
}
