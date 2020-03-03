use crate::config::MQTTConfiguration;
use crate::providers::observers::{Observation, Observer, ObserverInfo};
use crate::utils::DuckResult;
use rumq_client::{self, MqttOptions, QoS, Request};
use std::collections::HashSet;
use std::iter::FromIterator;
use tokio::runtime::Builder;

pub struct MqttObserver {
    options: MQTTConfiguration,
    info: ObserverInfo,
}

impl MqttObserver {
    pub fn new(config: &MQTTConfiguration) -> Self {
        MqttObserver {
            options: config.clone(),
            info: ObserverInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    None => true,
                    Some(e) => e,
                },
                collectors: match &config.collectors {
                    Option::None => Option::None,
                    Option::Some(collectors) => {
                        Some(HashSet::from_iter(collectors.iter().cloned()))
                    }
                },
            },
        }
    }
}

impl Observer for MqttObserver {
    fn info(&self) -> &ObserverInfo {
        &self.info
    }

    fn observe(&self, observation: Observation) -> DuckResult<()> {
        if let Observation::BuildStatusChanged(build) = observation {
            let mut rt = Builder::new().basic_scheduler().build()?;

            rt.block_on(async {
                let options = MqttOptions::new(
                    "duck-mqtt-client",
                    &self.options.broker.hostname,
                    self.options.broker.port,
                );
                let (mut tx, rx) = tokio::sync::mpsc::channel(1);
                let _event_loop = rumq_client::eventloop(options, rx);

                let msg = format!(
                    "Hi! Build {} has status {:?}",
                    build.definition_name, build.status
                );

                tx.send(Request::Publish(rumq_client::publish(
                    &self.options.topic,
                    QoS::AtLeastOnce,
                    msg,
                )))
                .await
            })?;
        }

        Ok(())
    }
}
