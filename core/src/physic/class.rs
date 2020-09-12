use crate::util::RcCell;
use m::{fx, Fx};
use ncollide3d::pipeline::{self, CollisionGroups, CollisionObjectSlabHandle, GeometricQueryType};
use std::f64::consts::PI;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysicTeam {
    Team1,
    Team2,
    Team3,
    Team4,
}

impl PhysicTeam {
    fn to_groups(&self) -> usize {
        return match self {
            PhysicTeam::Team1 => GROUPS_TEAM1,
            PhysicTeam::Team2 => GROUPS_TEAM2,
            PhysicTeam::Team3 => GROUPS_TEAM3,
            PhysicTeam::Team4 => GROUPS_TEAM4,
        };
    }

    fn to_excepted_groups(&self) -> [usize; 3] {
        let mut member_groups = [0usize; 3];
        let mut index = 0usize;
        if *self != PhysicTeam::Team1 {
            member_groups[index] = GROUPS_TEAM1;
            index += 1;
        }
        if *self != PhysicTeam::Team2 {
            member_groups[index] = GROUPS_TEAM2;
            index += 1;
        }
        if *self != PhysicTeam::Team3 {
            member_groups[index] = GROUPS_TEAM3;
            index += 1;
        }
        if *self != PhysicTeam::Team4 {
            member_groups[index] = GROUPS_TEAM3;
            index += 1;
        }
        return member_groups;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysicClass {
    Stage,
    NormalBounding,
    GiantBounding,
    GiantVolume,
    Target { team: PhysicTeam },
    Damage { team: PhysicTeam, stage: bool },
    Health { team: PhysicTeam, stage: bool },
    Defense { team: PhysicTeam },
    EnvDamage,
    EnvHealth,
}

const GROUPS_TEAM0: usize = 0; // world team
const GROUPS_TEAM1: usize = 1;
const GROUPS_TEAM2: usize = 2;
const GROUPS_TEAM3: usize = 3;
const GROUPS_TEAM4: usize = 4;

const GROUPS_STAGE: usize = 23;
const GROUPS_NORMAL_BOUNDING: usize = 24;
const GROUPS_GIANT_BOUNDING: usize = 24;
const GROUPS_GIANT_VOLUME: usize = 25;
const GROUPS_TARGET: usize = 26;
const GROUPS_DAMAGE: usize = 27;
const GROUPS_HEALTH: usize = 28;
const GROUPS_DEFENSE: usize = 29;

impl PhysicClass {
    pub fn to_geometric_query_type(&self) -> GeometricQueryType<Fx> {
        return match self {
            PhysicClass::Stage => GeometricQueryType::Contacts(fx(0), fx(PI / 36.0)),
            PhysicClass::NormalBounding => GeometricQueryType::Contacts(fx(0.1), fx(PI / 36.0)),
            PhysicClass::GiantBounding => GeometricQueryType::Contacts(fx(0.1), fx(PI / 36.0)),
            PhysicClass::GiantVolume => GeometricQueryType::Contacts(fx(0.1), fx(PI / 36.0)),
            PhysicClass::Target { team: _ } => GeometricQueryType::Proximity(fx(0.1)),
            PhysicClass::Damage { team: _, stage: _ } => GeometricQueryType::Proximity(fx(0.15)),
            PhysicClass::Health { team: _, stage: _ } => GeometricQueryType::Proximity(fx(0.15)),
            PhysicClass::Defense { team: _ } => GeometricQueryType::Proximity(fx(0.15)),
            PhysicClass::EnvDamage => GeometricQueryType::Proximity(fx(0.15)),
            PhysicClass::EnvHealth => GeometricQueryType::Proximity(fx(0.15)),
        };
    }

    pub fn to_collision_groups(&self) -> CollisionGroups {
        return match self {
            PhysicClass::Stage => Self::groups_stage(),
            PhysicClass::NormalBounding => Self::groups_normal_bounding(),
            PhysicClass::GiantBounding => Self::groups_giant_bounding(),
            PhysicClass::GiantVolume => Self::groups_giant_volume(),
            PhysicClass::Target { team } => Self::groups_target(team),
            PhysicClass::Damage { team, stage } => Self::groups_damage(team, *stage),
            PhysicClass::Health { team, stage } => Self::groups_health(team, *stage),
            PhysicClass::Defense { team } => Self::groups_defense(team),
            PhysicClass::EnvDamage => Self::groups_env_damage(),
            PhysicClass::EnvHealth => Self::groups_env_health(),
        };
    }

    fn groups_stage() -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_STAGE, GROUPS_TEAM0]);
        groups.set_whitelist(&[
            GROUPS_NORMAL_BOUNDING,
            GROUPS_GIANT_BOUNDING,
            GROUPS_DAMAGE,
            GROUPS_HEALTH,
        ]);
        return groups;
    }

    fn groups_normal_bounding() -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_NORMAL_BOUNDING]);
        groups.set_whitelist(&[GROUPS_STAGE, GROUPS_GIANT_VOLUME]);
        return groups;
    }

    fn groups_giant_bounding() -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_GIANT_BOUNDING]);
        groups.set_whitelist(&[GROUPS_STAGE]);
        return groups;
    }

    fn groups_giant_volume() -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_GIANT_VOLUME]);
        groups.set_whitelist(&[GROUPS_STAGE, GROUPS_NORMAL_BOUNDING]);
        return groups;
    }

    fn groups_target(team: &PhysicTeam) -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_TARGET, team.to_groups()]);
        groups.set_whitelist(&[GROUPS_DAMAGE, GROUPS_HEALTH]);
        return groups;
    }

    pub fn groups_damage(team: &PhysicTeam, stage: bool) -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_DAMAGE, team.to_groups()]);
        groups.set_blacklist(&[team.to_groups()]);
        if stage {
            groups.set_whitelist(&[GROUPS_TARGET, GROUPS_DEFENSE, GROUPS_STAGE]);
        } else {
            groups.set_whitelist(&[GROUPS_TARGET, GROUPS_DEFENSE]);
        }
        return groups;
    }

    pub fn groups_health(team: &PhysicTeam, stage: bool) -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_HEALTH, team.to_groups()]);
        groups.set_blacklist(&team.to_excepted_groups());
        if stage {
            groups.set_whitelist(&[GROUPS_TARGET, GROUPS_STAGE]);
        } else {
            groups.set_whitelist(&[GROUPS_TARGET]);
        }
        return groups;
    }

    pub fn groups_defense(team: &PhysicTeam) -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_DEFENSE, team.to_groups()]);
        groups.set_blacklist(&[team.to_groups()]);
        groups.set_whitelist(&[GROUPS_DAMAGE]);
        return groups;
    }

    pub fn groups_env_damage() -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_DAMAGE]);
        groups.set_whitelist(&[GROUPS_TARGET, GROUPS_DEFENSE]);
        return groups;
    }

    pub fn groups_env_health() -> CollisionGroups {
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[GROUPS_HEALTH]);
        groups.set_whitelist(&[GROUPS_TARGET]);
        return groups;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groups_stage() {}
}
