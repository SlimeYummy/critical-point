use std::collections::HashMap;
use ncollide3d::shape::ShapeHandle;
use crate::utils::Fx;

struct ModelRepository {
    repository: HashMap<String, ShapeHandle<Fx>>,
}

impl ModelRepository {
    pub fn new() -> ModelRepository {
        return ModelRepository{
            repository: HashMap::new(),
        };
    }
}
