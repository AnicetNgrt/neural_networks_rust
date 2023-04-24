#[cfg(feature = "Scalar")]
pub type Scalar = Scalar;

#[cfg(not(feature = "Scalar"))]
pub type Scalar = f32;

#[cfg(feature = "nalgebra")]
pub mod nalgebra_matrix;

#[cfg(feature = "nalgebra")]
pub use nalgebra_matrix::Matrix;

#[cfg(feature = "linalg-rayon")]
pub mod rayon_matrix;
#[cfg(all(feature = "linalg-rayon", not(feature = "nalgebra")))]
pub use rayon_matrix::Matrix;

#[cfg(feature = "linalg")]
pub mod matrix;
#[cfg(all(feature = "linalg", not(feature = "nalgebra"), not(feature = "linalg-rayon")))]
pub use matrix::Matrix;

#[cfg(feature = "faer")]
pub mod faer_matrix;
#[cfg(all(feature = "faer", not(feature = "linalg"), not(feature = "nalgebra"), not(feature = "linalg-rayon")))]
pub use faer_matrix::Matrix;

pub trait MatrixTrait: Clone {
    fn zeros(nrow: usize, ncol: usize) -> Self;

    /// Creates a matrix with random values between min and max (excluded).
    fn random_uniform(nrow: usize, ncol: usize, min: Scalar, max: Scalar) -> Self;

    /// Creates a matrix with random values following a normal distribution.
    fn random_normal(nrow: usize, ncol: usize, mean: Scalar, std_dev: Scalar) -> Self;

    /// Fills the matrix with the iterator columns after columns by chunking the data by n_rows.
    /// ```txt
    /// Your data : [[col1: row0 row1 ... rowNrow][col2]...[colNcol]]
    /// Result :
    /// [
    ///    [col0: row0 row1 ... rowNrow],
    ///    [col1: row0 row1 ... rowNrow],
    ///     ...
    ///    [colNcol: row0 row1 ... rowNrow],
    /// ]
    /// ```
    fn from_iter(nrow: usize, ncol: usize, data: impl Iterator<Item = Scalar>) -> Self;

    /// ```txt
    /// Your data :
    /// [
    ///    [row0: col0 col1 ... colNcol],
    ///    [row1: col0 col1 ... colNcol],
    ///     ...
    ///    [rowNrow: col0 col1 ... colNcol],
    /// ]
    /// 
    /// Result :
    /// [
    ///    [col0: row0 row1 ... rowNrow],
    ///    [col1: row0 row1 ... rowNrow],
    ///     ...
    ///    [colNcol: row0 row1 ... rowNrow],
    /// ]
    /// ```
    fn from_row_leading_matrix(m: &Vec<Vec<Scalar>>) -> Self;

    fn from_column_leading_matrix(m: &Vec<Vec<Scalar>>) -> Self;

    /// fills a column vector row by row with values of index 0 to v.len()
    fn from_column_vector(v: &Vec<Scalar>) -> Self;

    /// fills a row vector column by column with values of index 0 to v.len()
    fn from_row_vector(v: &Vec<Scalar>) -> Self;

    fn from_fn<F>(nrows: usize, ncols: usize, f: F) -> Self
    where
        F: FnMut(usize, usize) -> Scalar;

    fn columns_map(&self, f: impl Fn(usize, &Vec<Scalar>) -> Vec<Scalar>) -> Self;

    fn get_column(&self, index: usize) -> Vec<Scalar>;

    fn get_row(&self, index: usize) -> Vec<Scalar>;

    fn map(&self, f: impl Fn(Scalar) -> Scalar + Sync) -> Self;

    fn map_indexed_mut(&mut self, f: impl Fn(usize, usize, Scalar) -> Scalar + Sync) -> &mut Self;

    fn dot(&self, other: &Self) -> Self;

    fn columns_sum(&self) -> Vec<Scalar>;

    fn component_mul(&self, other: &Self) -> Self;

    fn component_add(&self, other: &Self) -> Self;

    fn component_sub(&self, other: &Self) -> Self;

    fn component_div(&self, other: &Self) -> Self;

    fn transpose(&self) -> Self;

    fn get_data(&self) -> Vec<Vec<Scalar>>;

    fn get_data_row_leading(&self) -> Vec<Vec<Scalar>>;
    
    /// returns the dimensions of the matrix (nrow, ncol)
    fn dim(&self) -> (usize, usize);

    fn scalar_add(&self, scalar: Scalar) -> Self;

    fn scalar_mul(&self, scalar: Scalar) -> Self;

    fn scalar_sub(&self, scalar: Scalar) -> Self;

    fn scalar_div(&self, scalar: Scalar) -> Self;

    fn index(&self, row: usize, col: usize) -> Scalar;

    fn index_mut(&mut self, row: usize, col: usize) -> &mut Scalar;
}