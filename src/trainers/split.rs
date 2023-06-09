use crate::{
    benchmarking::{EpochEvaluation, ModelEvaluation, TrainingEvaluation},
    linalg::Scalar,
    model::Model,
    monitor::TM,
    network::params::NetworkParams,
    vec_utils::r2_score_vector2,
};

#[cfg(feature = "data")]
use crate::datatable::DataTable;

#[cfg(not(feature = "data"))]
use rand::thread_rng;
#[cfg(not(feature = "data"))]
use rand::seq::SliceRandom;


pub type ReporterClosure = dyn FnMut(usize, EpochEvaluation) -> ();

pub struct SplitTraining {
    pub ratio: Scalar,
    pub real_time_reporter: Option<Box<ReporterClosure>>,
    pub model: Option<NetworkParams>,
    pub all_epochs_validation: bool,
    pub all_epochs_r2: bool,
}

impl SplitTraining {
    pub fn new(ratio: Scalar) -> Self {
        Self {
            ratio,
            real_time_reporter: None,
            all_epochs_validation: false,
            all_epochs_r2: false,
            model: None,
        }
    }

    pub fn take_model(&mut self) -> NetworkParams {
        self.model.take().unwrap()
    }

    /// Enables computing the R2 score of the model at the end of each epoch
    /// and reporting it if a real time reporter is attached.
    ///
    /// /!\ Requires `all_epochs_validation` to be enabled.
    ///
    /// /!\ Is time consuming.
    ///
    /// Otherwise computes it only at the end of the final epoch
    pub fn all_epochs_r2(&mut self) -> &mut Self {
        self.all_epochs_r2 = true;
        self
    }

    /// Enables computing the validation score of the model at the end of each epoch
    /// and reporting it if a real time reporter is attached.
    ///
    /// /!\ Is time consuming.
    ///
    /// Otherwise computes it only at the end of the final epoch
    pub fn all_epochs_validation(&mut self) -> &mut Self {
        self.all_epochs_validation = true;
        self
    }

    /// Attaches a real time reporter to the trainer.
    ///
    /// The reporter is a closure that takes as arguments:
    /// - the current epoch
    /// - the evaluation of the current epoch
    ///
    pub fn attach_real_time_reporter<F>(&mut self, reporter: F) -> &mut Self
    where
        F: FnMut(usize, EpochEvaluation) -> () + 'static,
    {
        self.real_time_reporter = Some(Box::new(reporter));
        self
    }

    #[cfg(feature = "data")]
    pub fn run(&mut self, model: &Model, data: &DataTable) -> (DataTable, ModelEvaluation) {
        assert!(!self.all_epochs_r2 || self.all_epochs_validation);
        
        TM::start("split");

        TM::start("init");

        let mut preds_and_ids = DataTable::new_empty();
        let mut model_eval = ModelEvaluation::new_empty();

        let predicted_features = model.dataset_config.predicted_features_names();
        let id_column = model
            .dataset_config
            .get_id_column()
            .expect("One feature must be configurationified as an id in the dataset dataset_config.");
        let mut network = model.to_network();

        // Split the data between validation and training
        let (train_table, validation) = data.split_ratio(self.ratio);

        // Shuffle the validation and training set and split it between x and y
        let (validation_x_table, validation_y_table) =
            validation.random_order_in_out(&predicted_features);

        // Convert the validation set to vectors
        let validation_x = validation_x_table.drop_column(id_column).to_vectors();
        let validation_y = validation_y_table.to_vectors();

        TM::end_with_message(format!(
            "Initialized training with {} samples\nInitialized validation with {} samples",
            train_table.num_rows(),
            validation_x_table.num_rows()
        ));

        TM::start("epochs");

        let mut eval = TrainingEvaluation::new_empty();
        let epochs = model.epochs;
        for e in 0..epochs {
            TM::start(&format!("{}/{}", e + 1, epochs));

            let train_loss = model.train_epoch(e, &mut network, &train_table, id_column);

            let loss_fn = model.loss.to_loss();
            let (preds, loss_avg, loss_std) = if e == model.epochs - 1 || self.all_epochs_validation
            {
                let vloss = network.predict_evaluate_many(
                    &validation_x,
                    &validation_y,
                    &loss_fn,
                    model.batch_size.unwrap_or(validation_x.len()),
                );
                vloss
            } else {
                (vec![], -1.0, -1.0)
            };

            let r2 = if e == model.epochs - 1 || self.all_epochs_r2 {
                TM::start("r2");
                let r2 = r2_score_vector2(&validation_y, &preds);
                TM::end_with_message(format!("R2: {}", r2));
                r2
            } else {
                -1.0
            };

            let epoch_eval = EpochEvaluation::new(train_loss, loss_avg, loss_std, r2);

            // Report the benchmark in real time if expected
            if let Some(reporter) = self.real_time_reporter.as_mut() {
                reporter(e, epoch_eval.clone());
            }

            // Save the predictions if it is the last epoch
            if e == model.epochs - 1 {
                preds_and_ids = preds_and_ids.apppend(
                    &DataTable::from_vectors(&predicted_features, &preds)
                        .add_column_from(&validation_x_table, id_column),
                );
            };

            TM::end_with_message(format!("Training Loss: {}\n ", train_loss));

            eval.add_epoch(epoch_eval);
        }
        TM::end_with_message(format!("Final performance: {:#?}", eval.get_final_epoch()));

        model_eval.add_fold(eval);
        self.model = Some(network.get_params());

        TM::end();

        (preds_and_ids, model_eval)
    }

    #[cfg(not(feature = "data"))]
    pub fn run(&mut self, model: &Model, data_x: &Vec<Vec<Scalar>>, data_y: &Vec<Vec<Scalar>>) -> (Vec<Vec<Scalar>>, ModelEvaluation) {
        assert!(!self.all_epochs_r2 || self.all_epochs_validation);
        assert!(data_x.len() == data_y.len());
        
        TM::start("split");

        TM::start("init");

        let mut model_eval = ModelEvaluation::new_empty();
        let mut network = model.to_network(data_x[0].len());
        
        // Split the data between validation and training
        let split_at = (self.ratio * data_x.len() as Scalar) as usize;
        let mut ids = (0..data_x.len()).map(|x| x as Scalar).collect::<Vec<_>>();
        ids.shuffle(&mut thread_rng());

        let data_x = ids.iter().map(|&i| data_x[i as usize].clone()).collect::<Vec<_>>();
        let data_y = ids.iter().map(|&i| data_y[i as usize].clone()).collect::<Vec<_>>();

        let (train_x, validation_x) = data_x.split_at(split_at);
        let (train_y, validation_y) = data_y.split_at(split_at);

        let train_x = train_x.to_vec();
        let train_y = train_y.to_vec();
        let validation_x = validation_x.to_vec();
        let validation_y = validation_y.to_vec();

        TM::end_with_message(format!(
            "Initialized training with {} samples\nInitialized validation with {} samples",
            train_x.len(),
            validation_x.len()
        ));

        TM::start("epochs");

        let mut final_predictions = vec![];

        let mut eval = TrainingEvaluation::new_empty();
        let epochs = model.epochs;
        for e in 0..epochs {
            TM::start(&format!("{}/{}", e + 1, epochs));

            let train_loss = model.train_epoch(e, &mut network, &train_x, &train_y);

            let loss_fn = model.loss.to_loss();
            let (preds, loss_avg, loss_std) = if e == model.epochs - 1 || self.all_epochs_validation
            {
                let vloss = network.predict_evaluate_many(
                    &validation_x,
                    &validation_y,
                    &loss_fn,
                    model.batch_size.unwrap_or(validation_x.len()),
                );
                vloss
            } else {
                (vec![], -1.0, -1.0)
            };

            let r2 = if e == model.epochs - 1 || self.all_epochs_r2 {
                TM::start("r2");
                let r2 = r2_score_vector2(&validation_y, &preds);
                TM::end_with_message(format!("R2: {}", r2));
                r2
            } else {
                -1.0
            };

            let epoch_eval = EpochEvaluation::new(train_loss, loss_avg, loss_std, r2);

            // Report the benchmark in real time if expected
            if let Some(reporter) = self.real_time_reporter.as_mut() {
                reporter(e, epoch_eval.clone());
            }

            // Save the predictions if it is the last epoch
            if e == model.epochs - 1 {
                final_predictions = preds.clone();
            };

            TM::end_with_message(format!("Training Loss: {}\n ", train_loss));

            eval.add_epoch(epoch_eval);
        }
        TM::end_with_message(format!("Final performance: {:#?}", eval.get_final_epoch()));
        
        model_eval.add_fold(eval);
        self.model = Some(network.get_params());

        // reorder predictions
        let mut reordered_predictions = Vec::with_capacity(data_x.len());
        for i in 0..data_x.len() {
            reordered_predictions[ids[i] as usize] = final_predictions[i].clone();
        }

        TM::end();

        (reordered_predictions, model_eval)
    }
}
