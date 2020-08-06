mod base;
mod character;
mod command;
mod engine;
mod lerper;
mod operation;
mod stage;

pub use crate::derive::LogicObjX;
pub use base::{LogicObj, LogicObjStatic, LogicObjSuper, NewContext, StateContext, UpdateContext};
pub use character::*;
pub use command::*;
pub use engine::*;
pub use operation::*;
pub use stage::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate as core;
    use crate::id::{ObjID, CLASS_CHARACTER, CLASS_STAGE};
    use crate::state::{StateBus, StateLifecycle, StateLocalReg, StateOwnerX, StateRef};
    use m::fx;
    use na::{Point3, Vector2, Isometry3};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(StateOwnerX, Debug)]
    #[class_id(CLASS_STAGE)]
    struct OwnerStage {
        obj_id: ObjID,
        lifecycle: StateLifecycle,
        state: StateRef<StateStage>,
    }

    impl OwnerStage {
        fn new(bus: Rc<RefCell<StateBus>>) -> OwnerStage {
            return OwnerStage {
                obj_id: ObjID::from(100000),
                lifecycle: StateLifecycle::Created,
                state: StateRef::<StateStage>::new(ObjID::from(100000), StateLocalReg::new(bus)),
            };
        }
    }

    #[derive(StateOwnerX, Debug)]
    #[class_id(CLASS_CHARACTER)]
    struct OwnerCharacter {
        obj_id: ObjID,
        lifecycle: StateLifecycle,
        state: StateRef<StateCharacter>,
    }

    impl OwnerCharacter {
        fn new(bus: Rc<RefCell<StateBus>>) -> OwnerCharacter {
            return OwnerCharacter {
                obj_id: ObjID::from(100001),
                lifecycle: StateLifecycle::Created,
                state: StateRef::<StateCharacter>::new(
                    ObjID::from(100001),
                    StateLocalReg::new(bus),
                ),
            };
        }
    }

    #[test]
    fn test_logic_all() {
        let mut engine = LogicEngine::new(fx(1.0 / 20.0));
        let bus = Rc::new(RefCell::new(StateBus::new()));

        let stage = Box::new(OwnerStage::new(bus.clone()));
        stage.state.register().unwrap();
        let chara = Box::new(OwnerCharacter::new(bus.clone()));
        chara.state.register().unwrap();

        // tick 1
        engine.command(&Command::NewStage(CmdNewStage {})).unwrap();
        engine
            .command(&Command::NewCharacter(CmdNewCharacter {
                position: Point3::new(fx(0), fx(0.1), fx(0)),
                direction: Vector2::new(fx(0), fx(1)),
                speed: fx(0.5),
                is_main: true,
            }))
            .unwrap();
        let pool = engine.tick().unwrap();
        bus.borrow_mut().dispatch(pool);

        assert_eq!(stage.state.state().unwrap().obj_id, ObjID::from(100000));
        assert_eq!(
            stage.state.state().unwrap().lifecycle,
            StateLifecycle::Created
        );
        assert_eq!(chara.state.state().unwrap().obj_id, ObjID::from(100001));
        assert_eq!(
            chara.state.state().unwrap().lifecycle,
            StateLifecycle::Created
        );
        assert_eq!(
            chara.state.state().unwrap().isometry,
            Isometry3::new(na::zero(), na::zero()),
            // Point3::new(fx(0), fx(0.1), fx(1.0 / 20.0 * 0.5))
        );

        // tick 2
        engine
            .command(&Command::MoveCharacter(CmdMoveCharacter {
                obj_id: ObjID::from(100001),
                direction: Vector2::new(fx(10), fx(0)),
                is_moving: true,
            }))
            .unwrap();
        let pool = engine.tick().unwrap();
        bus.borrow_mut().dispatch(pool);

        assert_eq!(stage.state.state().unwrap().obj_id, ObjID::from(100000));
        assert_eq!(
            stage.state.state().unwrap().lifecycle,
            StateLifecycle::Updated
        );
        assert_eq!(chara.state.state().unwrap().obj_id, ObjID::from(100001));
        assert_eq!(
            chara.state.state().unwrap().lifecycle,
            StateLifecycle::Updated
        );
        assert_eq!(
            chara.state.state().unwrap().isometry,
            Isometry3::new(na::zero(), na::zero()),
            // Point3::new(fx(1.0 / 20.0 * 0.5), fx(0.1), fx(1.0 / 20.0 * 0.5))
        );
    }
}
