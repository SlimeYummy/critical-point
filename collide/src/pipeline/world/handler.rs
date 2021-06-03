use crate::pipeline::object::{CollisionObjectSet, CollisionObjectSlab, CollisionObjectSlabHandle};
use math::Fx;
use ncollide3d::pipeline::{BroadPhaseInterferenceHandler, InteractionGraph, NarrowPhase};

pub struct CollisionWorldInterferenceHandler<'a, 'b, T> {
    pub(crate) narrow_phase: &'b mut NarrowPhase<Fx, CollisionObjectSlabHandle>,
    pub(crate) interactions: &'b mut InteractionGraph<Fx, CollisionObjectSlabHandle>,
    pub(crate) objects: &'a CollisionObjectSlab<T>,
}

impl<'a, 'b, T> BroadPhaseInterferenceHandler<CollisionObjectSlabHandle>
    for CollisionWorldInterferenceHandler<'a, 'b, T>
{
    fn is_interference_allowed(
        &mut self,
        b1: &CollisionObjectSlabHandle,
        b2: &CollisionObjectSlabHandle,
    ) -> bool {
        let co1 = match self.objects.collision_object(*b1) {
            Some(val) => val,
            None => return false,
        };
        let co2 = match self.objects.collision_object(*b2) {
            Some(val) => val,
            None => return false,
        };

        return (b1 != b2)
            & co1
                .collision_groups()
                .can_interact_with(co2.collision_groups());
    }

    fn interference_started(
        &mut self,
        b1: &CollisionObjectSlabHandle,
        b2: &CollisionObjectSlabHandle,
    ) {
        self.narrow_phase
            .handle_interaction(self.interactions, self.objects, *b1, *b2, true);
    }

    fn interference_stopped(
        &mut self,
        b1: &CollisionObjectSlabHandle,
        b2: &CollisionObjectSlabHandle,
    ) {
        self.narrow_phase
            .handle_interaction(&mut self.interactions, self.objects, *b1, *b2, false);
    }
}
