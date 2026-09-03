use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Service, Stop, StopTime, Trip};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimetableIndex {
    pub generated_at: DateTime<Utc>,
    pub stations: HashMap<String, Vec<String>>,
    pub stops: HashMap<String, Stop>,
    pub trips: HashMap<String, Trip>,
    pub stop_times: HashMap<String, Vec<StopTime>>,
    pub services: HashMap<String, Service>,
}
