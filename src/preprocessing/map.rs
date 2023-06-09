use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    dataset::{Dataset, Feature},
    datatable::DataTable,
    linalg::Scalar,
};

use super::{feature_cached::FeatureExtractorCached, DataTransformation, CachedConfig};

#[derive(Default, Serialize, Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
pub enum MapSelector {
    #[default]
    All,
    Equal(MapValue),
}

impl MapSelector {
    pub fn all() -> Self {
        Self::All
    }

    pub fn equal(value: MapValue) -> Self {
        Self::Equal(value)
    }

    pub fn equal_scalar(value: Scalar) -> Self {
        Self::Equal(MapValue::scalar(value))
    }

    pub fn find_all_corresponding(&self, data: &DataTable, column: &str) -> Vec<(Scalar, bool)> {
        let mut values = Vec::new();

        match self {
            MapSelector::All => {
                let column = data.column_to_vector(column);
                for value in &column {
                    values.push((*value, true));
                }
            }
            MapSelector::Equal(value) => {
                let value = value.find_all_corresponding(data);
                let column = data.column_to_vector(column);
                for (value, column) in value.iter().zip(column.iter()) {
                    if value == column {
                        values.push((*value, true));
                    } else {
                        values.push((*value, false));
                    }
                }
            }
        }

        values
    }
}

#[derive(Default, Serialize, Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
pub enum MapOp {
    #[default]
    None,
    ReplaceWith(MapValue),
}

impl MapOp {
    pub fn replace_with(value: MapValue) -> Self {
        Self::ReplaceWith(value)
    }

    pub fn replace_with_scalar(value: Scalar) -> Self {
        Self::ReplaceWith(MapValue::scalar(value))
    }

    pub fn replace_with_feature<S: ToString>(feature_name: S) -> Self {
        Self::ReplaceWith(MapValue::take_from_feature(feature_name))
    }

    pub fn apply(&self, data: &DataTable, corresponding_in: Vec<(Scalar, bool)>) -> Vec<Scalar> {
        let mut values = Vec::new();

        match self {
            MapOp::None => {
                for (value, _) in corresponding_in {
                    values.push(value);
                }
            }
            MapOp::ReplaceWith(value) => {
                let value = value.find_all_corresponding(data);
                for (value, corresponding) in value.iter().zip(corresponding_in.iter()) {
                    if corresponding.1 {
                        values.push(*value);
                    } else {
                        values.push(corresponding.0);
                    }
                }
            }
        }

        values
    }
}
 
#[derive(Default, Serialize, Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
pub enum MapValue {
    #[default]
    Zero,
    ConstantScalar(String),
    Feature(String),
}

impl MapValue {
    pub fn zero() -> Self {
        Self::Zero
    }

    pub fn scalar(value: Scalar) -> Self {
        Self::ConstantScalar(value.to_string())
    }

    pub fn take_from_feature<S: ToString>(feature_name: S) -> Self {
        Self::Feature(feature_name.to_string())
    }

    pub fn find_all_corresponding(&self, data: &DataTable) -> Vec<Scalar> {
        let mut values = Vec::new();

        match self {
            MapValue::Zero => {
                for _ in 0..data.num_rows() {
                    values.push(0.0);
                }
            }
            MapValue::ConstantScalar(value) => {
                let value = value.parse::<Scalar>().unwrap();
                for _ in 0..data.num_rows() {
                    values.push(value);
                }
            }
            MapValue::Feature(feature) => {
                values = data.column_to_vector(feature);
            }
        }

        values
    }
}

pub struct Map {
    mapped_features: HashMap<String, (MapSelector, MapOp)>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            mapped_features: HashMap::new(),
        }
    }
}

impl DataTransformation for Map {
    fn transform(
        &mut self,
        cached_config: &CachedConfig,
        dataset_config: &Dataset,
        data: &DataTable,
    ) -> (Dataset, DataTable) {
        let mut mapped_features = HashMap::new();

        for feature in dataset_config.features.iter() {
            if let Some(map) = &feature.mapped {
                mapped_features.insert(feature.name.clone(), map.clone());
            }
        }

        self.mapped_features = mapped_features.clone();

        let mut extractor = FeatureExtractorCached::new(
            Box::new(move |feature: &Feature| match &feature.mapped {
                Some(_) => {
                    let mut feature = feature.clone();
                    feature.mapped = None;
                    Some(feature)
                }
                _ => None,
            }),
            Box::new(
                move |data: &DataTable, extracted: &Feature, feature: &Feature| {
                    let corresponding_in = mapped_features[&feature.name]
                        .0
                        .find_all_corresponding(data, &feature.name);

                    let out = mapped_features[&feature.name]
                        .1
                        .apply(data, corresponding_in);

                    data.drop_column(&feature.name)
                        .with_column_scalar(&feature.name, &out)
                        .rename_column(&feature.name, &extracted.name)
                },
            ),
        );

        extractor.transform(cached_config, dataset_config, data)
    }

    fn reverse_columnswise(&mut self, data: &DataTable) -> DataTable {
        // not all maps are invertible
        // and some maps are invertible but the user may not want to reverse them
        // so it is kinda out of scope for now
        data.clone()
    }

    fn get_name(&self) -> String {
        "map".to_string()
    }
}
