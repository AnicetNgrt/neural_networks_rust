use serde::{Serialize, Deserialize};

use crate::linalg::Matrix;
use crate::linalg::MatrixTrait;

pub mod mse;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub enum Losses {
    MSE,
}

impl Losses {
    pub fn to_loss(&self) -> Loss {
        match self {
            Losses::MSE => mse::new(),
        }
    }
}

pub type LossFn = fn(&Matrix, &Matrix) -> f64;
pub type LossPrimeFn = fn(&Matrix, &Matrix) -> Matrix;

pub struct Loss {
    loss: LossFn,
    derivative: LossPrimeFn,
}

impl Loss {
    pub fn new(loss: LossFn, derivative: LossPrimeFn) -> Self {
        Self { loss, derivative }
    }
}

impl Loss {
    pub fn loss(&self, y_true: &Matrix, y_pred: &Matrix) -> f64 {
        (self.loss)(y_true, y_pred)
    }

    pub fn loss_prime(&self, y_true: &Matrix, y_pred: &Matrix) -> Matrix {
        (self.derivative)(y_true, y_pred)
    }

    pub fn loss_vec(&self, y_true: &Vec<Vec<f64>>, y_pred: &Vec<Vec<f64>>) -> f64 {
        let y_true = Matrix::from_row_leading_matrix(&y_true);
        let y_pred = Matrix::from_row_leading_matrix(&y_pred);
        self.loss(&y_true, &y_pred)
    }
}
