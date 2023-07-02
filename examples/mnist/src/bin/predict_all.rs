use neural_networks_rust::{
    datatable::DataTable,
    model::Model,
    network::params::NetworkParams,
    preprocessing::{attach_ids::AttachIds, Pipeline},
};

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_name = &args[1];
    let weights_file = &args[2];

    let mut model = Model::from_json_file(format!("models/{}.json", config_name));

    let mut pipeline = Pipeline::basic_single_pass();
    let (updated_dataset_spec, data) = pipeline
        .push(AttachIds::new("id"))
        .load_data_and_spec("./dataset/train.csv", &model.dataset)
        .run();

    println!("data: {:#?}", data);

    let model = model.with_new_dataset(updated_dataset_spec);
    let out_features = model.dataset.out_features_names();

    let (x_table, _) = data.random_order_in_out(&out_features);

    let x = x_table.drop_column("id").to_vectors();

    let weights = NetworkParams::from_json(format!("models_weights/{}.json", weights_file));
    let mut network = model.to_network();
    network.load_params(&weights);

    let preds = network.predict_many(&x, model.batch_size.unwrap_or(x.len()));

    let preds_and_ids =
        DataTable::from_vectors(&out_features, &preds).add_column_from(&x_table, "id");

    let preds_and_ids = pipeline.revert(&preds_and_ids);
    let data = pipeline.revert(&data);
    
    let values = data.select_columns(&["id", "label"]);
    let data = data.drop_columns(&["label", "label-confidence"]);
    let values_and_preds = values.inner_join(&preds_and_ids, "id", "id", Some("pred"));

    data.to_csv_file(format!("models_stats/{}_data_for_values_and_preds.csv", weights_file));
    values_and_preds.to_csv_file(format!("models_stats/{}_values_and_preds.csv", weights_file));

    println!("{:#?}", values_and_preds);
}
