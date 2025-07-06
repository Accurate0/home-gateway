use super::{Message, TemperatureSensorHandler, TemperatureSensorHandlerBuilder};
use crate::{settings::Settings, types::SharedActorState};
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, queues, routing},
};

pub async fn spawn_temperature_sensor_handler(
    root_supervisor_ref: &ActorRef<()>,
    shared_actor_state: SharedActorState,
    settings: Settings,
) -> anyhow::Result<()> {
    let door_handler_factory_def = Factory::<
        (),
        Message,
        (),
        TemperatureSensorHandler,
        routing::QueuerRouting<(), Message>,
        queues::DefaultQueue<(), Message>,
    >::default();

    let door_handler_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(TemperatureSensorHandlerBuilder {
            shared_actor_state,
            temperature_sensor_settings: settings.temperature_sensors,
        }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(1)
        .build();

    let (_, _) = root_supervisor_ref
        .spawn_linked(
            Some(TemperatureSensorHandler::NAME.to_string()),
            door_handler_factory_def,
            door_handler_factory_args,
        )
        .await?;

    Ok(())
}
