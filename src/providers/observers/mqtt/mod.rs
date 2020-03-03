use crate::providers::observers::{Observer, ObserverInfo, Observation};
use crate::utils::DuckResult;

pub struct MqttObserver {}

impl Observer for MqttObserver {
    fn info(&self) -> &ObserverInfo {
        unimplemented!()
    }
    fn observe(&self, observation: Observation) -> DuckResult<()> {
        unimplemented!()
    }
}
