# Neural Networks in Rust

Implementing Neural Networks in Rust from scratch + utils for data manipulation.

This was made for the purpose of my own learning. It is obviously not a production-ready library by any means. 

Feel free to give feedback.

## Include

Add this in your project's `Cargo.toml` file:

```toml
[dependencies]
neural_networks_rust = "*"
```

## Code example

Simple regression workflow example:

```rust
// Loading a model specification from JSON
let mut model = Model::from_json_file("my_model_spec.json");

// Applying a data pipeline on it according to its dataset specification
let mut pipeline = Pipeline::basic_single_pass();
let (updated_dataset_spec, data) = pipeline
    .add(AttachIds::new("id"))
    .run("./dataset", &model.dataset);

let model = model.with_new_dataset(updated_dataset_spec);

// Training it using k-fold cross validation
// + extracting test & training metrics per folds & per epochs
// + extracting all predictions made during final epoch
let kfold = model.trainer.maybe_kfold().expect("We only do k-folds here!");
let (validation_preds, model_eval) = kfold
    .attach_real_time_reporter(|report| println!("Perf report: {:#?}", report))
    .run(&model, &data);

// Reverting the pipeline on the predictions & data to get interpretable values
let validation_preds = pipeline.revert_columnswise(&validation_preds);
let data = pipeline.revert_columnswise(&data);

// Joining the data and the predictions together
let data_and_preds = data.inner_join(&validation_preds, "id", "id", Some("pred"));

// Saving it all to disk
data_and_preds.to_file("my_model_preds.csv");
model_eval.to_json_file("my_model_evals.json");
```

You can then plot the results using a third-party crate like `gnuplot` *(recommended)*, `plotly` or even `plotters`.

But first you would need to write or generate your model's specification.

Here is an example generating it with code *(recommended)*:

```rust
// Including all features from some CSV dataset
let mut dataset_spec = Dataset::from_csv("kc_house_data.csv");
dataset_spec
    // Removing useless features for both the model & derived features
    .remove_features(&["id", "zipcode", "sqft_living15", "sqft_lot15"])
    // Setting up the price as the "output" predicted feature
    .add_opt_to("price", Out)
    // Setting up the date format
    .add_opt_to("date", DateFormat("%Y%m%dT%H%M%S"))
    // Converting the date to a date_timestamp feature
    .add_opt_to("date", AddExtractedTimestamp)
    // Excluding the date from the model
    .add_opt_to("date", Not(&UsedInModel))
    // Mapping yr_renovated to yr_built if = to 0
    .add_opt_to(
        "yr_renovated",
        Mapped(
            MapSelector::Equal(0.0.into()),
            MapOp::ReplaceWith(MapValue::Feature("yr_built".to_string())),
        ),
    )
    // Converting relevant features to their log10
    .add_opt(Log10.only(&["sqft_living", "sqft_above", "price"]))
    // Adding ^2 features of all input features 
    // (including the added ones like the timestamp)
    .add_opt(AddSquared.except(&["price", "date"]).incl_added_features())
    // Filtering rows according to feature's outliers
    .add_opt(FilterOutliers.except(&["date"]).incl_added_features())
    // Normalizing everything
    .add_opt(Normalized.except(&["date"]).incl_added_features());

// Creating our layers
let h_size = dataset_spec.in_features_names().len() + 1;
let nh = 8;

let mut layers = vec![];
for i in 0..nh {
    layers.push(LayerSpec::from_options(&[
        OutSize(h_size),
        Activation(ReLU),
        Optimizer(adam()),
    ]));
}
let final_layer = LayerSpec::from_options(&[
    OutSize(1),
    Activation(Linear),
    Optimizer(adam()),
]);

// Putting it all together
let model = Model::from_options(&[
    Dataset(dataset_spec),
    HiddenLayers(layers.as_slice()),
    FinalLayer(final_layer),
    BatchSize(128),
    Trainer(Trainers::KFolds(8)),
    Epochs(300),
]);

// Saving it all
model.to_json_file("my_model_spec.json");
```

## Working features

### Neural Networks

- Neural Network abstraction
- Layers
    - Shared abstraction
        - Forward/Backward passes
        - Handle (mini)batches
    - Dense layers
        - Weights optimizer/initializer (no regularization yet)
        - Biases optimizer/initializer (no regularization yet)
    - Activation layers
        - Linear
        - Hyperbolic Tangent
        - ReLU
        - Sigmoid
        - Tanh
    - Full layers (Dense + Activation)
        - Optional Dropout regularization
- Stochastic Gradient Descent (SGD)
    - Optional optimizers
        - Momentum
        - Adam
    - Learning rate scheduling
        - Constant
        - Piecewise constant
        - Inverse time decay
- Initializers
    - Zeros
    - Random uniform (signed & unsigned)
    - Glorot uniform
- Losses
    - MSE

### Models utils

- Abstractions over training
    - KFolds + model benchmarking in a few LoC
- Model specification
    - Simple API to create as code
    - From/to JSON conversion
    - Specify training parameters (k-folds, epochs...)
    - Specify the dataset
        - Features in/out
        - Features preprocessing
- Model benchmarking
    - Handy utilities to store & compute training metrics

### Preprocessing

- Layer-based data pipelines abstraction
    - Cached & revertible feature mapping/feature extraction layers
        - Normalize
        - Square
        - Log10 scale
        - Extract month from Date string
        - Extract unix timestamp from Date string
        - Limited row mapping (for advanced manipulations)
    - Row filtering layers
        - Filter outliers
    - Other layers
        - Attach IDs column

### Data analysis

- Simple Polars DataFrame wrapper (DataTable)
    - Load & manipulate CSV data
    - Generate input/output vectors
        - Sample
        - Shuffle
        - Split
        - K-folds
- `Vector<f64>` statistics & manipulation utils
    - Stats (mean, std dev, correlation, quartiles...)
    - Normalization

### Linear algebra

- Many backends for the Matrix type (toggled using compile-time cargo features)
    - Feature `nalgebra` (enabled by default)
        - Fast
        - CPU-bound
    - Feature `linalg`
        - 100% custom Vec-based
        - Slow
        - CPU-bound
    - Feature `linalg-rayon`
        - linalg parallelized with rayon
        - Way faster than linalg but slower than nalgebra
        - CPU-bound
    - Feature `faer`
        - WIP
        - Should be fast but for now it's on par with linalg-rayon
        - CPU-bound
- Switching from `f32` (default) to `f64`-backed `Scalar` type with the `f64` feature