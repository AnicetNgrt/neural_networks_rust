#[cfg(feature = "arrayfire")]
use arrayfire::print;

#[allow(unused_imports)]
#[allow(unused_variables)]

use jiro_nn::vision::{
    conv_layer::avg_pooling_layer::AvgPoolingLayer, image::Image, image::ImageTrait,
    image_layer::ImageLayer,
};

pub fn main() {
    let image = Image::random_normal(6, 6, 3, 2, 2.0, 1.0);

    #[cfg(feature = "arrayfire")]
    print(&image.0);

    let mut layer = AvgPoolingLayer::new(2);

    #[allow(unused_variables)]
    let image = layer.forward(image);

    #[cfg(feature = "arrayfire")]
    print(&image.0);

    let gradient = Image::random_normal(3, 3, 3, 2, 0.0, 0.5);

    #[cfg(feature = "arrayfire")]
    print(&gradient.0);

    #[allow(unused_variables)]
    let gradient = layer.backward(0, gradient);

    #[cfg(feature = "arrayfire")]
    print(&gradient.0);
}
