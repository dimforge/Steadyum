use crate::runner::RunnerCommand;
use flume::Sender;
use steadyum_api_types::zenoh::ZenohContext;

use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::messages::{PartitionnerMessage, RunnerMessage, PARTITIONNER_QUEUE};
use steadyum_api_types::simulation::SimulationBounds;
use zenoh::prelude::sync::SyncResolve;
use zenoh::prelude::SplitBuffer;

pub fn start_command_loop(
    bounds: SimulationBounds,
    commands_snd: Sender<RunnerCommand>,
) -> anyhow::Result<()> {
    /*
     * Init S3
     */
    let mut kvs = KvsContext::new()?;

    /*
     * Init zenoh
     */
    let zenoh = ZenohContext::new()?;
    let runner_key = bounds.zenoh_queue_key();
    let queue = zenoh
        .session
        .declare_subscriber(&runner_key)
        .res_sync()
        .expect("Commands error.");
    println!("Waiting for messages. Press Ctrl-C to exit.");
    zenoh.put_json(
        PARTITIONNER_QUEUE,
        &PartitionnerMessage::AckStart { origin: runner_key },
    )?;

    while let Ok(sample) = queue.recv() {
        let payload = sample.value.payload.contiguous();
        let body = String::from_utf8_lossy(&payload);
        // println!("Runner processing message message: {}", body);
        let message: RunnerMessage = serde_json::from_str(&body).unwrap();
        match message {
            RunnerMessage::AssignJoint(joint) => {
                commands_snd.send(RunnerCommand::AssignJoint(joint))?;
            }
            RunnerMessage::ReAssignObject { uuid, warm_object } => {
                let cold_object = kvs.get_cold_object(uuid).expect("E");
                commands_snd.send(RunnerCommand::CreateBody {
                    uuid,
                    cold_object,
                    warm_object,
                })?;
            }
            RunnerMessage::MoveObject { uuid, position } => {
                commands_snd.send(RunnerCommand::MoveBody { uuid, position })?;
            }
            RunnerMessage::UpdateColdObject { uuid } => {
                commands_snd.send(RunnerCommand::UpdateColdObject { uuid })?;
            }
            RunnerMessage::StartStop { running } => {
                commands_snd.send(RunnerCommand::StartStop { running })?;
            }
            RunnerMessage::RunSteps {
                curr_step,
                num_steps,
            } => {
                commands_snd.send(RunnerCommand::RunSteps {
                    curr_step,
                    num_steps,
                })?;
            }
        }
    }
    dbg!("OUT!");

    Ok(())
}
