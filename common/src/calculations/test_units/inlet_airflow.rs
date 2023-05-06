use crate::{calculations::ScalesWith, impl_UnitMath};
use std::ops::{Add, Div, Mul, Neg, Sub};

use super::fan_diameter::FanDiameter;

#[derive(Clone, PartialEq, Debug, Copy)]
pub struct InletAirflow(f64);
impl_UnitMath!(InletAirflow);

impl InletAirflow {
    // inlet airflow - A, outlet airlow - F, Induced flow -D
    pub fn new(cfm: f64) -> InletAirflow {
        InletAirflow(cfm)
    }

    pub fn from_cfm(cfm: f64) -> InletAirflow {
        InletAirflow(cfm)
    }

    pub fn cfm(&self) -> f64 {
        self.0
    }
}

impl ScalesWith<FanDiameter> for InletAirflow {
    fn scale(self, &from: &FanDiameter, &to: &FanDiameter) -> Self {
        Self(self.0 * (to / from).powi(3))
    }
}
