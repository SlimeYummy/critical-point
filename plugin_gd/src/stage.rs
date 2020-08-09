use crate::core_ex::{GdSyncCore, SyncStateRef};
use crate::util::{NodeExt, ResultExt};
use core::agent::SyncStateReg;
use core::id::{ObjID, CLASS_STAGE};
use core::logic::StateStage;
use core::state::StateOwnerX;
use core::util::try_result;
use gdnative::prelude::*;

#[derive(StateOwnerX, NativeClass)]
#[inherit(Spatial)]
#[user_data(LocalCellData<GdStage>)]
#[class_id(CLASS_STAGE)]
pub struct GdStage {
    obj_id: ObjID,
    state: SyncStateRef<StateStage>,
}

#[methods]
impl GdStage {
    fn new(_: &Spatial) -> GdStage {
        godot_print!("GdStage::new()");

        let obj_id = ObjID::from(100000);
        return GdStage {
            obj_id,
            state: SyncStateRef::new(obj_id, SyncStateReg::default()),
        };
    }

    #[export]
    fn _ready(&mut self, owner: &Spatial) {
        godot_print!("GdStage::_ready()");

        try_result(|| {
            let core =
                unsafe { owner.root_instance_ref::<GdSyncCore, Node, _>("./Root/SyncCore")? };
            let agent = core.map_mut(|core, _| core.get_agent()).cast_err()?;
            self.state.set_reg(SyncStateReg::new(&agent))?;
            self.state.register()
        })
        .expect("GdStage::_ready()");
    }
}
