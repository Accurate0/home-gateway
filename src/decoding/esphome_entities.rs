use serde::Deserialize;

use crate::integrations::esphome::{EsphomeDomain, EsphomeTarget};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EsphomeEntities {
    #[serde(default)]
    pub sensor: Vec<String>,
    #[serde(default)]
    pub binary_sensor: Vec<String>,
    #[serde(default)]
    pub light: Vec<String>,
}

impl EsphomeEntities {
    pub fn is_empty(&self) -> bool {
        self.sensor.is_empty() && self.binary_sensor.is_empty() && self.light.is_empty()
    }

    pub fn targets<'a>(&'a self, node: &'a str) -> impl Iterator<Item = EsphomeTarget> + 'a {
        let domains = [
            (EsphomeDomain::Sensor, &self.sensor),
            (EsphomeDomain::BinarySensor, &self.binary_sensor),
            (EsphomeDomain::Light, &self.light),
        ];

        domains.into_iter().flat_map(move |(domain, object_ids)| {
            object_ids.iter().map(move |object_id| EsphomeTarget {
                node: node.to_owned(),
                domain,
                object_id: object_id.clone(),
            })
        })
    }

    pub fn light(&self) -> Option<&str> {
        match self.light.as_slice() {
            [object_id] => Some(object_id),
            _ => None,
        }
    }
}
