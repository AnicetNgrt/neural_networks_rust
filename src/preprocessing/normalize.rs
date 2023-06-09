use std::collections::HashMap;

use crate::{
    dataset::{Dataset, Feature},
    datatable::DataTable,
};

use super::{feature_cached::FeatureExtractorCached, DataTransformation, CachedConfig};
use crate::linalg::Scalar;

pub struct Normalize {
    pub features_min_max: HashMap<String, (Scalar, Scalar)>,
}

impl Normalize {
    pub fn new() -> Self {
        Self {
            features_min_max: HashMap::new(),
        }
    }

    pub fn same_normalization(&mut self, new_feature: &str, old_feature: &str) -> &mut Self {
        let min_max = self.features_min_max.get(old_feature).unwrap();
        self.features_min_max
            .insert(new_feature.to_string(), *min_max);
        self
    }

    pub fn denormalize_data(&self, data: &DataTable) -> DataTable {
        let mut denormalized_data = data.clone();

        for (feature_name, min_max) in self.features_min_max.iter() {
            if denormalized_data.has_column(feature_name) {
                denormalized_data = denormalized_data.denormalize_column(feature_name, *min_max);
            }
        }

        denormalized_data
    }
}

impl DataTransformation for Normalize {
    fn transform(
        &mut self,
        cached_config: &CachedConfig,
        dataset_config: &Dataset,
        data: &DataTable,
    ) -> (Dataset, DataTable) {
        let mut features_min_max: HashMap<String, (Scalar, Scalar)> = HashMap::new();

        for feature in dataset_config.features.iter() {
            if feature.normalized {
                let min_max = data.min_max_column(&feature.name);
                features_min_max.insert(feature.name.clone(), min_max);
            }
        }

        self.features_min_max = features_min_max.clone();

        let mut extractor = FeatureExtractorCached::new(
            Box::new(move |feature: &Feature| match &feature.with_normalized {
                Some(new_feature) => Some(*new_feature.clone()),
                _ => match &feature.normalized {
                    true => {
                        let mut feature = feature.clone();
                        feature.normalized = false;
                        Some(feature)
                    }
                    _ => None,
                },
            }),
            Box::new(
                move |data: &DataTable, extracted: &Feature, feature: &Feature| {
                    data.normalize_column(
                        &feature.name,
                        *features_min_max.get(&feature.name).unwrap(),
                    )
                    .rename_column(&feature.name, &extracted.name)
                },
            ),
        );

        extractor.transform(cached_config, dataset_config, data)
    }

    fn reverse_columnswise(&mut self, data: &DataTable) -> DataTable {
        self.denormalize_data(data)
    }

    fn get_name(&self) -> String {
        "norm".to_string()
    }
}
