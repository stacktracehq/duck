use crate::config::MQTTConfiguration;
use crate::providers::observers::{Observation, Observer, ObserverInfo};
use crate::utils::DuckResult;
use log;
use rumq_client::{self, MqttOptions, QoS, Request};
use std::collections::HashSet;
use std::iter::FromIterator;
use tokio::runtime::Builder;
use tokio::time::{self, Duration};

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
        log::info!("MqttObserver got an observation ({:?})...", observation);
        if let Observation::BuildStatusChanged(build) = observation {
            log::info!(
                "MqttObserver would like to publish something about ({}, {:?})...",
                build.definition_name,
                build.status
            );
            let mut rt = Builder::new().basic_scheduler().enable_all().build()?;

            rt.block_on(async {
                let mut options = MqttOptions::new(
                    "duck-mqtt-client",
                    &self.options.broker.hostname,
                    self.options.broker.port,
                );
                options.set_credentials(
                    &self.options.credentials.username,
                    &self.options.credentials.password,
                );

                log::info!("MqttObserver is about to start the event loop");

                let (mut tx, rx) = tokio::sync::mpsc::channel(1);
                let _event_loop = rumq_client::eventloop(options, rx);

                let msg = format!(
                    "Hi! Build {} has status {:?}",
                    build.definition_name, build.status
                );

                log::info!("MqttObserver is about to send {}", msg);

                let result = tx
                    .send(Request::Publish(rumq_client::publish(
                        &self.options.topic,
                        QoS::AtLeastOnce,
                        msg,
                    )))
                    .await;

                time::delay_for(Duration::new(30, 0)).await;

                log::info!("MqttObserver finished sending... Hooray?");
                result
            })?;
        }

        Ok(())
    }
}
