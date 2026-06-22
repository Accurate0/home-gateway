use std::collections::HashMap;

use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    types::SharedActorState,
};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub struct NewEvent {
    pub event_id: Uuid,
    /// esphome node name, used as the plant sensor's settings key.
    pub node: String,
    /// esphome sensor object_id that produced this reading (e.g. `soil_moisture`).
    pub object_id: String,
    pub value: f64,
}

pub enum Message {
    NewEvent(NewEvent),
}

#[derive(Default)]
pub struct PlantSensorState {
    /// `(node, action index) -> condition satisfied at the last reading`. Used to
    /// fire workflows only on the rising edge of a threshold being crossed.
    last_satisfied: HashMap<(String, usize), bool>,
}

pub struct PlantSensorHandler {
    shared_actor_state: SharedActorState,
}

impl PlantSensorHandler {
    pub const NAME: &str = "plant-sensor";

    fn execute_workflow(
        event_id: Uuid,
        workflow: &crate::settings::workflow::WorkflowSettings,
    ) -> Result<(), anyhow::Error> {
        let Some(actor) = ractor::registry::where_is(WorkflowWorker::NAME.to_string()) else {
            tracing::warn!("actor not found for workflow");
            return Ok(());
        };

        actor.send_message(FactoryMessage::Dispatch(Job {
            key: (),
            msg: WorkflowWorkerMessage::Execute {
                event_id,
                workflow: workflow.to_owned(),
            },
            options: JobOptions::default(),
            accepted: None,
        }))?;

        Ok(())
    }

    fn handle(&self, message: Message, state: &mut PlantSensorState) -> Result<(), anyhow::Error> {
        let Message::NewEvent(event) = message;

        let settings = self.shared_actor_state.settings.load();
        let Some(plant_settings) = settings.plant_sensors.get(&event.node) else {
            tracing::warn!("no plant sensor setting found for: {}", event.node);
            return Ok(());
        };

        for (index, action) in plant_settings.actions.iter().enumerate() {
            if action.entity != event.object_id {
                continue;
            }

            let satisfied = action.when.is_satisfied(event.value);
            let key = (event.node.clone(), index);
            let was_satisfied = state.last_satisfied.insert(key, satisfied).unwrap_or(false);

            // rising edge only: fire when the threshold is newly crossed
            if satisfied && !was_satisfied {
                tracing::info!(
                    "plant sensor {} {} reading {} crossed {:?}",
                    plant_settings.id,
                    event.object_id,
                    event.value,
                    action.when,
                );
                Self::execute_workflow(event.event_id, &action.workflow)?;
            }
        }

        Ok(())
    }
}

impl Worker for PlantSensorHandler {
    type Key = ();
    type Message = Message;
    type State = PlantSensorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(PlantSensorState::default())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg, state) {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct PlantSensorHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}

impl WorkerBuilder<PlantSensorHandler, ()> for PlantSensorHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (PlantSensorHandler, ()) {
        (
            PlantSensorHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
