use crate::vision::{image::ImageTrait, image::Image};
use super::ConvActivationLayer;

pub fn new() -> ConvActivationLayer {
    ConvActivationLayer::new(
        |m| m.clone(), 
        |m| Image::constant(
            m.image_dims().0,
            m.image_dims().1,
            m.image_dims().2,
            m.samples(),
            1.,
        )
    )
}