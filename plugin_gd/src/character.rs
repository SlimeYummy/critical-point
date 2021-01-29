use crate::core_ex::{RES_CACHE, SYNC_AGENT};
use anyhow::Result;
use core::character::StateCharaGeneral;
use core::id::ObjID;
use core::state::StateRef;
use euclid::Vector3D;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Spatial)]
#[register_with(register_properties)]
#[user_data(LocalCellData<CharaGeneral>)]
pub struct CharaGeneral {
    obj_id: ObjID,
    state: StateRef<StateCharaGeneral>,
    counter: u32,
}

fn register_properties(builder: &ClassBuilder<CharaGeneral>) {
    builder
        .add_property::<String>("critical_point/obj_id")
        .with_default(ObjID::invalid().into())
        .with_getter(|app: &CharaGeneral, _| app.obj_id.clone().into())
        .with_setter(|app: &mut CharaGeneral, _, val: String| app.obj_id = ObjID::from(val))
        .done();
}

#[methods]
impl CharaGeneral {
    fn new(_: &Spatial) -> CharaGeneral {
        godot_print!("CharaGeneral::new()");

        return CharaGeneral {
            obj_id: ObjID::invalid(),
            state: StateRef::invaild(),
            counter: 0,
        };
    }

    #[export]
    fn _ready(&mut self, owner: &Spatial) {
        godot_print!("CharaGeneral::_ready()");
        owner.set_physics_process(true);

        let result: Result<()> = try {
            let fobj_id = RES_CACHE().get_fobj_id(&self.obj_id)?;
            let binder = SYNC_AGENT().new_binder();
            self.state.start(fobj_id, binder)?;
        };
        if let Err(err) = result {
            godot_error!("CharaGeneral::_ready() => {:?}", err);
        }
    }

    #[export]
    fn _physics_process(&mut self, owner: &Spatial, _delta: f64) {
        let result: Result<()> = try {
            let state = self.state.state()?;

            owner.set_translation(Vector3D::new(
                state.position.translation.vector.x.to_f32(),
                state.position.translation.vector.y.to_f32(),
                state.position.translation.vector.z.to_f32(),
            ));

            self.counter += 1;
            if self.counter % 60 == 0 {
                godot_print!("{:?}", state.position);
            }
        };
        if let Err(err) = result {
            godot_error!("CharaGeneral::_physics_process() => {:?}", err);
        }

        // let state = self.state.state().unwrap();

        // let model = unsafe { owner.typed_node_tref::<Spatial, _>("./Model").unwrap() };
        // let mat = state
        //     .isometry
        //     .rotation
        //     .to_rotation_matrix()
        //     .matrix()
        //     .clone();
        // // godot_print!("===== {} ===== {} {} {}", state.isometry.rotation, x, y, z);
        // model.set_transform(Transform {
        //     basis: Basis::from_elements([
        //         Vector3D::new(
        //             mat[(0, 0)].to_f32(),
        //             mat[(1, 0)].to_f32(),
        //             mat[(2, 0)].to_f32(),
        //         ),
        //         Vector3D::new(
        //             mat[(0, 1)].to_f32(),
        //             mat[(1, 1)].to_f32(),
        //             mat[(2, 1)].to_f32(),
        //         ),
        //         Vector3D::new(
        //             mat[(0, 2)].to_f32(),
        //             mat[(1, 2)].to_f32(),
        //             mat[(2, 2)].to_f32(),
        //         ),
        //     ]),
        //     origin: Vector3D::zero(),
        // });

        // self.update_camera(owner).unwrap();
    }

    // #[export]
    // fn _input(&mut self, _owner: &Spatial, event: Ref<InputEvent>) {
    //     let event = unsafe { event.assume_safe() };
    //     if let Some(motion) = event.cast::<InputEventMouseMotion>() {
    //         let speed = motion.speed();

    //         self.camera_hori = self.camera_hori + speed.x * 0.0001;
    //         self.camera_hori = self.camera_hori % (2.0 * PI);

    //         self.camera_vert = self.camera_vert - speed.y * 0.0001;
    //         self.camera_vert = na::clamp(self.camera_vert, -FRAC_PI_2, FRAC_PI_2);
    //     }
    // }
}

// impl CharaGeneral {
//     pub fn move_character(&mut self, direction: Vector2D<f32, UnknownUnit>) {
//         let camera_rot: Rotation2<f32> = Rotation2::new(-self.camera_hori); // negative, right-handed
//         let move_dir: Vector2<f32> = camera_rot * Vector2::new(direction.x, direction.y);
//         if let Some(agent) = &self.agent {
//             // agent.operate(Operation::MoveCharacter(OpMoveCharacter {
//             //     direction: move_dir,
//             //     is_moving: true,
//             // }));
//         }
//     }

//     pub fn jump_character(&mut self) {
//         if let Some(agent) = &self.agent {
//             // agent.operate(Operation::JumpCharacter(OpJumpCharacter {}));
//         }
//     }
// }
