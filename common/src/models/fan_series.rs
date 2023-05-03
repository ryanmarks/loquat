use serde::{Deserialize, Serialize};

use super::fan_type::FanType;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct FanSeries {
    pub id: String,
    pub fan_type: FanType,
}
