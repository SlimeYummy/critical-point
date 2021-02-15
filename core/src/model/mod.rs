use std::collections::HashMap;
use crate::physics::ShapeHandle;
use crate::utils::Fx;

struct ModelRepository {
    repository: HashMap<String, ShapeHandle>,
}

impl ModelRepository {
    pub fn new() -> ModelRepository {
        return ModelRepository{
            repository: HashMap::new(),
        };
    }
}
