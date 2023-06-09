use std::fmt;

use crate::linalg::{MatrixTrait, Scalar};
use crate::{
    initializers::Initializers,
    layer::Layer,
    linalg::Matrix,
    optimizer::{Optimizers},
};

use super::LearnableLayer;

pub struct DenseLayer {
    // i inputs, j outputs, i x j connections
    input: Option<Matrix>,
    // j x i connection weights
    pub weights: Matrix,
    // j output biases (single column)
    pub biases: Matrix,
    weights_optimizer: Optimizers,
    biases_optimizer: Optimizers,
}

impl DenseLayer {
    pub fn new(
        i: usize,
        j: usize,
        weights_optimizer: Optimizers,
        biases_optimizer: Optimizers,
        weights_initializer: Initializers,
        biases_initializer: Initializers,
    ) -> Self {
        // about weights initialization : http://proceedings.mlr.press/v9/glorot10a/glorot10a.pdf

        let weights = weights_initializer.gen_matrix(j, i);
        let biases = biases_initializer.gen_vector(j);

        Self {
            weights: weights,
            biases: biases,
            input: None,
            weights_optimizer,
            biases_optimizer,
        }
    }
}

impl Layer for DenseLayer {
    /// `input` has shape `(i, n)` where `i` is the number of inputs and `n` is the number of samples.
    ///
    /// Returns output which has shape `(j, n)` where `j` is the number of outputs and `n` is the number of samples.
    fn forward(&mut self, input: Matrix) -> Matrix {
        // Y = W . X + B * (1...1)

        // println!("WEIGHTS");
        // self
        //     .weights.print();

        let res = self
            .weights
            .dot(&input)
            .component_add(&self.biases.dot(&Matrix::constant(1, input.dim().1, 1.0)));

        self.input = Some(input);
        res
    }

    /// `output_gradient` has shape `(j, n)` where `j` is the number of outputs and `n` is the number of samples.
    ///
    /// Returns `input_gradient` which has shape `(i, n)` where `i` is the number of inputs and `n` is the number of samples.
    fn backward(&mut self, epoch: usize, output_gradient: Matrix) -> Matrix {
        let input = self.input.as_ref().unwrap();

        let weights_gradient = &output_gradient.dot(&input.transpose());

        let biases_gradient = output_gradient.columns_sum();

        let input_gradient = self.weights.transpose().dot(&output_gradient);

        self.weights =
            self.weights_optimizer
                .update_parameters(epoch, &self.weights, &weights_gradient);
        self.biases =
            self.biases_optimizer
                .update_parameters(epoch, &self.biases, &biases_gradient);

        input_gradient
    }
}

impl LearnableLayer for DenseLayer {
    // returns a matrix of the (jxi) weights and the final column being the (j) biases
    fn get_learnable_parameters(&self) -> Vec<Vec<Scalar>> {
        let mut params = self.weights.get_data_col_leading();
        params.push(self.biases.get_column(0));
        params
    }

    // takes a matrix of the (jxi) weights and the final column being the (j) biases
    fn set_learnable_parameters(&mut self, params_matrix: &Vec<Vec<Scalar>>) {
        let mut weights = params_matrix.clone();
        let biases = weights.pop().unwrap();
        self.weights = Matrix::from_column_leading_vector2(&weights);
        self.biases = Matrix::from_column_vector(&biases);
    }
}

impl fmt::Debug for DenseLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dense Layer")
    }
}
