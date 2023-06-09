use serde::{Deserialize, Serialize};

use crate::{
    learning_rate::{default_learning_rate, LearningRateSchedule},
    linalg::Scalar,
    vision::{image::Image, image::ImageTrait},
};

use crate::optimizer::momentum::default_momentum;

// https://arxiv.org/pdf/1207.0580.pdf
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConvMomentum {
    #[serde(default = "default_momentum")]
    momentum: Scalar,
    #[serde(default = "default_learning_rate")]
    learning_rate: LearningRateSchedule,
    #[serde(skip)]
    v: Option<Image>,
}

impl ConvMomentum {
    pub fn new(learning_rate: LearningRateSchedule, momentum: Scalar) -> Self {
        Self {
            v: None,
            momentum,
            learning_rate,
        }
    }

    pub fn default() -> Self {
        Self {
            v: None,
            momentum: default_momentum(),
            learning_rate: default_learning_rate(),
        }
    }

    pub fn update_parameters(
        &mut self,
        epoch: usize,
        parameters: &Image,
        parameters_gradient: &Image,
    ) -> Image {
        let lr = self.learning_rate.get_learning_rate(epoch);

        if let None = &self.v {
            let (nrow, ncol, nchan) = parameters_gradient.image_dims();
            let n_sample = parameters_gradient.samples();
            self.v = Some(Image::zeros(nrow, ncol, nchan, n_sample));
        };

        let v = self.v.as_ref().unwrap();

        let v = v
            .scalar_mul(self.momentum)
            .component_add(&parameters_gradient.scalar_mul(lr));

        let new_params = parameters.component_sub(&v);
        self.v = Some(v);
        new_params
    }
}
