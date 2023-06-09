use std::fmt;

use arrayfire::{
    constant, exp, flip, index, join_many, maxof, mean, mean_all, minof, pow,
    random_normal, random_uniform, sign, sqrt, sum, sum_all, unwrap, wrap, Array, Dim4,
    RandomEngine, Seq, tile, convolve2_nn,
};
use rand::Rng;

use crate::linalg::{Matrix, MatrixTrait, Scalar};

use super::ImageTrait;

#[derive(Clone)]
pub struct Image(pub Array<Scalar>);

impl ImageTrait for Image {
    fn zeros(nrow: usize, ncol: usize, nchan: usize, samples: usize) -> Self {
        Self(Array::new(
            vec![0.0; nrow * ncol * nchan * samples].as_slice(),
            Dim4::new(&[
                nrow.try_into().unwrap(),
                ncol.try_into().unwrap(),
                nchan.try_into().unwrap(),
                samples.try_into().unwrap(),
            ]),
        ))
    }

    fn constant(nrow: usize, ncol: usize, nchan: usize, samples: usize, value: Scalar) -> Self {
        Self(constant!(
            value;
            nrow.try_into().unwrap(),
            ncol.try_into().unwrap(),
            nchan.try_into().unwrap(),
            samples.try_into().unwrap()
        ))
    }

    fn random_uniform(
        nrow: usize,
        ncol: usize,
        nchan: usize,
        samples: usize,
        min: Scalar,
        max: Scalar,
    ) -> Self {
        let mut rng = rand::thread_rng();
        Self(
            random_uniform::<Scalar>(
                Dim4::new(&[
                    nrow.try_into().unwrap(),
                    ncol.try_into().unwrap(),
                    nchan.try_into().unwrap(),
                    samples.try_into().unwrap(),
                ]),
                &RandomEngine::new(
                    arrayfire::RandomEngineType::MERSENNE_GP11213,
                    Some(rng.gen()),
                ),
            ) * (max - min)
                + constant!(min;
                    nrow.try_into().unwrap(),
                    ncol.try_into().unwrap(),
                    nchan.try_into().unwrap(),
                    samples.try_into().unwrap()
                ),
        )
    }

    fn random_normal(
        nrow: usize,
        ncol: usize,
        nchan: usize,
        samples: usize,
        mean: Scalar,
        stddev: Scalar,
    ) -> Self {
        let mut rng = rand::thread_rng();
        Self(
            random_normal::<Scalar>(
                Dim4::new(&[
                    nrow.try_into().unwrap(),
                    ncol.try_into().unwrap(),
                    nchan.try_into().unwrap(),
                    samples.try_into().unwrap(),
                ]),
                &RandomEngine::new(
                    arrayfire::RandomEngineType::MERSENNE_GP11213,
                    Some(rng.gen()),
                ),
            ) * stddev
                + constant!(mean;
                    nrow.try_into().unwrap(),
                    ncol.try_into().unwrap(),
                    nchan.try_into().unwrap(),
                    samples.try_into().unwrap()
                ),
        )
    }

    fn from_fn<F>(nrows: usize, ncols: usize, nchan: usize, samples: usize, mut f: F) -> Self
    where
        F: FnMut(usize, usize, usize, usize) -> Scalar,
    {
        let elements = (0..nrows * ncols * nchan * samples)
            .map(|i| {
                f(
                    (i / nrows) % ncols,
                    i % nrows,
                    (i / (nrows * ncols)) % nchan,
                    i / (nrows * ncols * nchan),
                )
            })
            .collect::<Vec<_>>();

        Self(Array::new_strided(
            elements.as_slice(),
            0,
            Dim4::new(&[
                nrows.try_into().unwrap(),
                ncols.try_into().unwrap(),
                nchan.try_into().unwrap(),
                samples.try_into().unwrap(),
            ]),
            Dim4::new(&[
                1,
                nrows.try_into().unwrap(),
                (nrows * ncols).try_into().unwrap(),
                (nrows * ncols * nchan).try_into().unwrap(),
            ]),
        ))
    }

    fn from_samples(samples: &Matrix, channels: usize) -> Self {
        let image_size = ((samples.dim().0 / channels) as f64).sqrt() as usize;
        let mut samples_flattened = vec![];
        for i in 0..samples.dim().1 {
            let mut sample = samples.get_column(i);
            samples_flattened.append(&mut sample);
        }

        Self(Array::new(
            samples_flattened.as_slice(),
            Dim4::new(&[
                image_size.try_into().unwrap(),
                image_size.try_into().unwrap(),
                channels.try_into().unwrap(),
                samples.dim().1.try_into().unwrap(),
            ]),
        ))
    }

    fn wrap(
        &self,
        ox: usize,
        oy: usize,
        wx: usize,
        wy: usize,
        sx: usize,
        sy: usize,
        px: usize,
        py: usize,
    ) -> Self {
        Self(wrap(
            &self.0,
            ox.try_into().unwrap(),
            oy.try_into().unwrap(),
            wx.try_into().unwrap(),
            wy.try_into().unwrap(),
            sx.try_into().unwrap(),
            sy.try_into().unwrap(),
            px.try_into().unwrap(),
            py.try_into().unwrap(),
            true,
        ))
    }

    fn unwrap(&self, wx: usize, wy: usize, sx: usize, sy: usize, px: usize, py: usize) -> Self {
        Self(unwrap(
            &self.0,
            wx.try_into().unwrap(),
            wy.try_into().unwrap(),
            sx.try_into().unwrap(),
            sy.try_into().unwrap(),
            px.try_into().unwrap(),
            py.try_into().unwrap(),
            true,
        ))
    }

    fn tile(
        &self,
        repetitions_row: usize,
        repetitions_col: usize,
        repetitions_chan: usize,
        repetition_sample: usize,
    ) -> Self {
        Self(tile(
            &self.0,
            Dim4::new(&[
                repetitions_row.try_into().unwrap(),
                repetitions_col.try_into().unwrap(),
                repetitions_chan.try_into().unwrap(),
                repetition_sample.try_into().unwrap(),
            ])
        ))
    }

    fn component_add(&self, other: &Self) -> Self {
        let samples = self.samples();
        let other_samples = other.samples();

        if samples == other_samples {
            Self(&self.0 + &other.0)
        } else {
            let mut res_samples = vec![];
            let other_sample = other.get_sample(0);
            for i in 0..samples {
                let sample = self.get_sample(i);
                res_samples.push(Self(sample.0 + &other_sample.0));
            }
            Self::join_samples(res_samples)
        }
    }

    fn component_sub(&self, other: &Self) -> Self {
        let samples = self.samples();
        let other_samples = other.samples();

        if samples == other_samples {
            Self(&self.0 - &other.0)
        } else {
            let mut res_samples = vec![];
            let other_sample = other.get_sample(0);
            for i in 0..samples {
                let sample = self.get_sample(i);
                res_samples.push(Self(sample.0 - &other_sample.0));
            }
            Self::join_samples(res_samples)
        }
    }

    fn component_mul(&self, other: &Self) -> Self {
        let samples = self.samples();
        let other_samples = other.samples();

        if samples == other_samples {
            Self(&self.0 * &other.0)
        } else {
            let mut res_samples = vec![];
            let other_sample = other.get_sample(0);
            for i in 0..samples {
                let sample = self.get_sample(i);
                res_samples.push(Self(sample.0 * &other_sample.0));
            }
            Self::join_samples(res_samples)
        }
    }

    fn component_div(&self, other: &Self) -> Self {
        let samples = self.samples();
        let other_samples = other.samples();

        if samples == other_samples {
            Self(&self.0 / &other.0)
        } else {
            let mut res_samples = vec![];
            let other_sample = other.get_sample(0);
            for i in 0..samples {
                let sample = self.get_sample(i);
                res_samples.push(Self(sample.0 / &other_sample.0));
            }
            Self::join_samples(res_samples)
        }
    }

    fn scalar_add(&self, scalar: Scalar) -> Self {
        Self(&self.0 + scalar)
    }

    fn scalar_sub(&self, scalar: Scalar) -> Self {
        Self(&self.0 - scalar)
    }

    fn scalar_mul(&self, scalar: Scalar) -> Self {
        Self(&self.0 * scalar)
    }

    fn scalar_div(&self, scalar: Scalar) -> Self {
        Self(&self.0 / scalar)
    }

    fn cross_correlate(&self, kernels: &Self) -> Self {
        let kernels = Self(flip(&flip(&kernels.0, 0), 1));
        self.convolve(&kernels, false)
    }

    fn convolve_full(&self, kernels: &Self) -> Self {
        self.convolve(&kernels, true)
    }

    fn flatten(&self) -> Matrix {
        let image_size = self.image_dims().0 * self.image_dims().1 * self.channels();
        Matrix::from_array(image_size, self.samples(), &self.0)
    }

    fn image_dims(&self) -> (usize, usize, usize) {
        (
            self.0.dims()[0] as usize,
            self.0.dims()[1] as usize,
            self.0.dims()[2] as usize,
        )
    }

    fn channels(&self) -> usize {
        self.0.dims()[2] as usize
    }

    fn samples(&self) -> usize {
        self.0.dims()[3] as usize
    }

    fn get_sample(&self, sample: usize) -> Self {
        Self(index(
            &self.0,
            &[
                Seq::default(),
                Seq::default(),
                Seq::default(),
                Seq::new(sample.try_into().unwrap(), sample.try_into().unwrap(), 1),
            ],
        ))
    }

    fn get_channel(&self, channel: usize) -> Self {
        Self(index(
            &self.0,
            &[
                Seq::default(),
                Seq::default(),
                Seq::new(channel.try_into().unwrap(), channel.try_into().unwrap(), 1),
                Seq::new(0, 0, 1),
            ],
        ))
    }

    fn get_channel_across_samples(&self, channel: usize) -> Self {
        Self(index(
            &self.0,
            &[
                Seq::default(),
                Seq::default(),
                Seq::new(channel.try_into().unwrap(), channel.try_into().unwrap(), 1),
                Seq::default(),
            ],
        ))
    }

    fn sum_samples(&self) -> Self {
        Self(sum(&self.0, 3))
    }

    fn join_channels(mut channels: Vec<Self>) -> Self {
        if channels.len() == 1 {
            return channels.pop().unwrap();
        }
        let mut res = join_many(2, vec![&channels[0].0, &channels[1].0]);
        for i in 2..channels.len() {
            res = join_many(2, vec![&res, &channels[i].0]);
        }
        Self(res)
    }

    fn join_samples(mut samples: Vec<Self>) -> Self {
        if samples.len() == 1 {
            return samples.pop().unwrap();
        }
        let mut res = join_many(3, vec![&samples[0].0, &samples[1].0]);
        for i in 2..samples.len() {
            res = join_many(3, vec![&res, &samples[i].0]);
        }
        Self(res)
    }

    fn square(&self) -> Self {
        Self(pow(
            &self.0,
            &constant!(2.0 as Scalar;
                self.image_dims().0.try_into().unwrap(),
                self.image_dims().1.try_into().unwrap(),
                self.channels().try_into().unwrap(),
                self.samples().try_into().unwrap()
            ),
            false,
        ))
    }

    fn sum(&self) -> Scalar {
        sum_all(&self.0).0
    }

    fn mean(&self) -> Scalar {
        mean_all(&self.0).0 as Scalar
    }

    fn mean_along(&self, dim: usize) -> Self {
        Self(mean(&self.0, dim.try_into().unwrap()))
    }

    fn exp(&self) -> Self {
        Self(exp(&self.0))
    }

    fn maxof(&self, other: &Self) -> Self {
        Self(maxof(&self.0, &other.0, false))
    }

    fn sign(&self) -> Self {
        Self(sign(&self.0)).scalar_mul(-2.0).scalar_add(1.0)
    }

    fn minof(&self, other: &Self) -> Self {
        Self(minof(&self.0, &other.0, false))
    }

    fn sqrt(&self) -> Self {
        Self(sqrt(&self.0))
    }
}

impl Image {
    fn convolve(&self, kernels: &Self, full: bool) -> Self {
        let (out_size, padding) = if full {
            (
                self.image_dims().0 + kernels.image_dims().0 - 1,
                Dim4::new(&[
                    (kernels.image_dims().0 - 1).try_into().unwrap(), 
                    (kernels.image_dims().1 - 1).try_into().unwrap(), 
                    1, 
                    1
                ]),
            )
        } else {
            (
                self.image_dims().0 - kernels.image_dims().0 + 1,
                Dim4::new(&[0, 0, 0, 0])
            )
        };

        let res = convolve2_nn(
            &self.0,
            &kernels.0,
            Dim4::new(&[1, 1, 1, 1]),
            padding,
            Dim4::new(&[0, 0, 0, 0])
        );

        let res = Self(index(
            &res,
            &[
                Seq::new(0, (out_size - 1).try_into().unwrap(), 1),
                Seq::new(0, (out_size - 1).try_into().unwrap(), 1),
                Seq::new(0, (kernels.samples() - 1).try_into().unwrap(), 1),
                Seq::new(0, (self.samples() - 1).try_into().unwrap(), 1),
            ],
        ));
        res
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("dims", &self.image_dims())
            .field("samples", &self.samples())
            .finish()
            .unwrap();
        self.flatten().fmt(f)
    }
}
