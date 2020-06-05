mod state_pool;
mod state_ref;

use crate::id::TypeID;

pub use state_pool::StatePool;
pub use state_ref::StateRef;

pub trait StateData {
    fn type_id() -> TypeID;
}