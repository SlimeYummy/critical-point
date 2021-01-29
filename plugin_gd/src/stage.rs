use crate::core_ex::{RES_CACHE, SYNC_AGENT};
use anyhow::Result;
use core::id::{FastObjID, ObjID};
use core::stage::StateStageGeneral;
use core::state::StateRef;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Spatial)]
#[register_with(register_properties)]
#[user_data(LocalCellData<StageGeneral>)]
pub struct StageGeneral {
    obj_id: ObjID,
    state: StateRef<StateStageGeneral>,
}

fn register_properties(builder: &ClassBuilder<StageGeneral>) {
    builder
        .add_property::<String>("critical_point/obj_id")
        .with_default(ObjID::invalid().into())
        .with_getter(|app: &StageGeneral, _| app.obj_id.clone().into())
        .with_setter(|app: &mut StageGeneral, _, val: String| app.obj_id = ObjID::from(val))
        .done();
}

#[methods]
impl StageGeneral {
    fn new(_: &Spatial) -> StageGeneral {
        godot_print!("StageGeneral::new()");

        return StageGeneral {
            obj_id: ObjID::invalid(),
            state: StateRef::invaild(),
        };
    }

    #[export]
    fn _ready(&mut self, owner: &Spatial) {
        godot_print!("StageGeneral::_ready() => call");
        owner.set_physics_process(true);

        let result: Result<()> = try {
            let fobj_id = RES_CACHE().get_fobj_id(&self.obj_id)?;
            let binder = SYNC_AGENT().new_binder();
            self.state.start(fobj_id, binder)?;
        };
        if let Err(err) = result {
            godot_error!("StageGeneral::_ready() => {:?}", err);
        }
    }

    #[export]
    fn _physics_process(&mut self, _owner: &Spatial, _delta: f64) {
        // if let Ok(state) = self.state.state() {
        //     godot_print!("{:?}", state);
        // } else {
        //     godot_print!("null");
        // }
    }
}
