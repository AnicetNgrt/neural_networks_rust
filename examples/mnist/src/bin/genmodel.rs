use neural_networks_rust::{
    dataset::{Dataset, FeatureOptions::*},
    model::ModelBuilder,
    trainers::Trainers, loss::Losses,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_name = &args[1];

    let mut dataset_spec = Dataset::from_csv("dataset/train.csv");
    dataset_spec
        .add_opt(Normalized.except(&["label"]))
        .add_opt_to("label", Out)
        .add_opt_to("label", OneHotEncode);

    let model = ModelBuilder::new(dataset_spec)
        .neural_network()
            .conv_network(1)
                .full_dense(32, 3)
                    .sigmoid()
                    .momentum()
                .end()
                .avg_pooling(2)
                .full_direct(5)
                    .sigmoid()
                    .momentum()
                .end()
                .avg_pooling(3)
            .end()
            .full_dense(128)
                .relu()
                .momentum()
            .end()
            .full_dense(10)
                .linear()
                .momentum()
            .end()
        .end()
        .epochs(10)
        .batch_size(128)
        .trainer(Trainers::SplitTraining(0.8))
        .loss(Losses::BCE)
        .build();

    //println!("{:#?}", model);

    model.to_json_file(format!("models/{}.json", config_name));
}
