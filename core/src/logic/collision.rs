use super::base::LogicTeam;
use super::LogicObj;
use crate::util::RcCell;
use m::Fx;
use ncollide3d::pipeline::{self, CollisionGroups, CollisionObjectSlabHandle};

//
// Types
//

pub type CollisionObject = pipeline::CollisionObject<Fx, RcCell<dyn LogicObj>>;
pub type CollisionHandle = CollisionObjectSlabHandle;

pub const INVAILD_COLLISION_HANDLE: CollisionHandle = CollisionObjectSlabHandle(usize::MAX);

//
// Groups
//

const COLLISION_MEMBER_STAGE: usize = 0;
const COLLISION_MEMBER_TEAM1: usize = 1;
const COLLISION_MEMBER_TEAM2: usize = 2;
const COLLISION_MEMBER_TEAM3: usize = 3;
const COLLISION_MEMBER_TEAM4: usize = 4;
const COLLISION_MEMBER_BOUNDING: usize = 20;
const COLLISION_MEMBER_BODY: usize = 21;
const COLLISION_MEMBER_DAMAGE: usize = 22;
const COLLISION_MEMBER_HEALTH: usize = 23;
const COLLISION_MEMBER_DEFENSE: usize = 24;

pub fn collision_groups_stage() -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[COLLISION_MEMBER_STAGE]);
    groups.set_whitelist(&[
        COLLISION_MEMBER_BOUNDING,
        COLLISION_MEMBER_DAMAGE,
        COLLISION_MEMBER_HEALTH,
    ]);
    return groups;
}

pub fn collision_groups_bounding(team: LogicTeam) -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[to_member(team), COLLISION_MEMBER_BOUNDING]);
    groups.set_blacklist(&[to_member(team)]);
    groups.set_whitelist(&[COLLISION_MEMBER_STAGE, COLLISION_MEMBER_BOUNDING]);
    return groups;
}

pub fn collision_groups_body(team: LogicTeam) -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[to_member(team), COLLISION_MEMBER_BODY]);
    groups.set_whitelist(&[COLLISION_MEMBER_DAMAGE, COLLISION_MEMBER_HEALTH]);
    return groups;
}

pub fn collision_groups_body_bounding(team: LogicTeam) -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[
        to_member(team),
        COLLISION_MEMBER_BODY,
        COLLISION_MEMBER_BOUNDING,
    ]);
    groups.set_whitelist(&[
        COLLISION_MEMBER_DAMAGE,
        COLLISION_MEMBER_HEALTH,
        COLLISION_MEMBER_STAGE,
    ]);
    return groups;
}

pub fn collision_groups_damage(team: LogicTeam, collision_stage: bool) -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[to_member(team), COLLISION_MEMBER_DAMAGE]);
    groups.set_blacklist(&[to_member(team)]);
    if !collision_stage {
        groups.set_whitelist(&[COLLISION_MEMBER_BODY, COLLISION_MEMBER_DEFENSE]);
    } else {
        groups.set_whitelist(&[
            COLLISION_MEMBER_BODY,
            COLLISION_MEMBER_DEFENSE,
            COLLISION_MEMBER_STAGE,
        ]);
    }
    return groups;
}

pub fn collision_groups_health(team: LogicTeam, collision_stage: bool) -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[to_member(team), COLLISION_MEMBER_HEALTH]);
    groups.set_blacklist(&without_member(team));
    if !collision_stage {
        groups.set_whitelist(&[COLLISION_MEMBER_BODY]);
    } else {
        groups.set_whitelist(&[COLLISION_MEMBER_BODY, COLLISION_MEMBER_STAGE]);
    }
    return groups;
}

pub fn collision_groups_defense(team: LogicTeam) -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[to_member(team), COLLISION_MEMBER_DEFENSE]);
    groups.set_blacklist(&[to_member(team)]);
    groups.set_whitelist(&[COLLISION_MEMBER_DAMAGE]);
    return groups;
}

pub fn collision_groups_env_damage() -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[COLLISION_MEMBER_DAMAGE]);
    groups.set_whitelist(&[COLLISION_MEMBER_BODY, COLLISION_MEMBER_DEFENSE]);
    return groups;
}

pub fn collision_groups_env_health() -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.set_membership(&[COLLISION_MEMBER_HEALTH]);
    groups.set_whitelist(&[COLLISION_MEMBER_BODY]);
    return groups;
}

fn to_member(team: LogicTeam) -> usize {
    return match team {
        LogicTeam::Team1 => COLLISION_MEMBER_TEAM1,
        LogicTeam::Team2 => COLLISION_MEMBER_TEAM2,
        LogicTeam::Team3 => COLLISION_MEMBER_TEAM3,
        LogicTeam::Team4 => COLLISION_MEMBER_TEAM4,
    };
}

fn without_member(team: LogicTeam) -> [usize; 3] {
    let mut member_groups = [0usize; 3];
    let mut index = 0usize;
    for iter in LogicTeam::Team1..=LogicTeam::Team4 {
        if iter != team {
            member_groups[index] = to_member(iter);
            index += 1;
        }
    }
    return member_groups;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groups_stage() {}
}
