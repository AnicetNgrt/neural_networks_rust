use jiro_nn::model::Model;
use jiro_nn::monitor::TM;
use jiro_nn::preprocessing::Pipeline;
use jiro_nn::preprocessing::attach_ids::AttachIds;
use jiro_nn::trainers::kfolds::KFolds;


pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_name = &args[1];

    let mut model = Model::from_json_file(format!("models/{}.json", config_name));

    let mut pipeline = Pipeline::basic_single_pass();
    let (updated_dataset_config, data) = pipeline
        .push(AttachIds::new("id"))
        .load_data("./dataset/kc_house_data.csv", Some(&model.dataset_config))
        .run();

    println!("data: {:#?}", data);

    let model = model.with_new_dataset(updated_dataset_config);
    
    TM::start_monitoring();

    let mut kfold = KFolds::new(4);
    let (preds_and_ids, model_eval) = kfold
        // .attach_real_time_reporter(|fold, epoch, report| {
        //     println!("Perf report: {:2} {:4} {:#?}", fold, epoch, report)
        // })
        .all_epochs_validation()
        .all_epochs_r2()
        .compute_best_model()
        // .compute_avg_model()
        .run(&model, &data);

    TM::stop_monitoring();

    let best_model_params = kfold.take_best_model();
    //let avg_model_params = kfold.take_avg_model();

    //best_model_params.to_json_file(format!("models_weights/{}_best_params.json", config_name));
    best_model_params.to_binary_compressed(format!("models_weights/{}_best_params.gz", config_name));
    //avg_model_params.to_json_file(format!("models_stats/{}_avg_params.json", config_name));

    let preds_and_ids = pipeline.revert(&preds_and_ids);
    let data = pipeline.revert(&data);
    let data_and_preds = data.inner_join(&preds_and_ids, "id", "id", Some("pred"));

    data_and_preds.to_csv_file(format!("models_stats/{}.csv", config_name));

    println!("{:#?}", data_and_preds);

    model_eval.to_json_file(format!("models_stats/{}.json", config_name));
}
