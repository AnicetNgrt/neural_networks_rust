use serde::{Deserialize, Serialize};

use crate::linalg::Matrix;

use self::{adam::Adam, momentum::Momentum, sgd::SGD};

pub mod adam;
pub mod momentum;
pub mod sgd;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Optimizers {
    SGD(SGD),
    Momentum(Momentum),
    Adam(Adam),
}

impl Optimizers {
    pub fn update_parameters(
        &mut self,
        epoch: usize,
        parameters: &Matrix,
        parameters_gradient: &Matrix,
    ) -> Matrix {
        match self {
            Optimizers::SGD(sgd) => sgd.update_parameters(epoch, parameters, parameters_gradient),
            Optimizers::Momentum(momentum) => {
                momentum.update_parameters(epoch, parameters, parameters_gradient)
            }
            Optimizers::Adam(adam) => {
                adam.update_parameters(epoch, parameters, parameters_gradient)
            }
        }
    }
}

pub fn adam() -> Optimizers {
    Optimizers::Adam(Adam::default())
}

pub fn sgd() -> Optimizers {
    Optimizers::SGD(SGD::default())
}

pub fn momentum() -> Optimizers {
    Optimizers::Momentum(Momentum::default())
}
