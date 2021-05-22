const GROUP_NONE: u8 = 0x01;
const GROUP_TEAM1: u8 = 0x02;
const GROUP_TEAM2: u8 = 0x04;
const GROUP_TEAM3: u8 = 0x08;
const GROUP_TEAM4: u8 = 0x10;
const GROUP_TEAM_ALL: u8 = 0x1F;

const GROUP_STAGE: u16 = 0x01;
const GROUP_VOLUME: u16 = 0x02;
const GROUP_GIANT_BOUNDING: u16 = 0x04;
const GROUP_GIANT_VOLUME: u16 = 0x08;
const GROUP_TARGET_BOX: u16 = 0x10;
const GROUP_HIT_BOX: u16 = 0x40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionTeam {
    None = 0,
    Team1 = 1,
    Team2 = 2,
    Team3 = 3,
    Team4 = 4,
}

impl Default for CollisionTeam {
    fn default() -> CollisionTeam {
        return CollisionTeam::None;
    }
}

impl From<u8> for CollisionTeam {
    fn from(team: u8) -> CollisionTeam {
        return match team {
            1 => CollisionTeam::Team1,
            2 => CollisionTeam::Team2,
            3 => CollisionTeam::Team3,
            4 => CollisionTeam::Team4,
            _ => CollisionTeam::None,
        };
    }
}

impl CollisionTeam {
    fn friend(&self) -> u8 {
        return match self {
            CollisionTeam::None => GROUP_NONE,
            CollisionTeam::Team1 => GROUP_TEAM1,
            CollisionTeam::Team2 => GROUP_TEAM2,
            CollisionTeam::Team3 => GROUP_TEAM3,
            CollisionTeam::Team4 => GROUP_TEAM4,
        };
    }

    fn enemy(&self) -> u8 {
        return match self {
            CollisionTeam::None => GROUP_TEAM_ALL,
            CollisionTeam::Team1 => GROUP_TEAM_ALL & !GROUP_TEAM1,
            CollisionTeam::Team2 => GROUP_TEAM_ALL & !GROUP_TEAM2,
            CollisionTeam::Team3 => GROUP_TEAM_ALL & !GROUP_TEAM3,
            CollisionTeam::Team4 => GROUP_TEAM_ALL & !GROUP_TEAM4,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionTeamFilter {
    None,
    All,
    Friend,
    Enemy,
}

impl Default for CollisionTeamFilter {
    fn default() -> CollisionTeamFilter {
        return CollisionTeamFilter::All;
    }
}

impl CollisionTeamFilter {
    pub fn new(friend: bool, enemy: bool) -> CollisionTeamFilter {
        return match (friend, enemy) {
            (false, false) => CollisionTeamFilter::None,
            (true, true) => CollisionTeamFilter::All,
            (true, false) => CollisionTeamFilter::Friend,
            (false, true) => CollisionTeamFilter::Enemy,
        };
    }
}

pub struct CollisionGroups {
    team_membership: u8,
    team_whitelist: u8,
    role_membership: u16,
    role_whitelist: u16,
}

impl CollisionGroups {
    #[inline]
    pub fn new_stage() -> CollisionGroups {
        return CollisionGroups {
            team_membership: GROUP_NONE,
            team_whitelist: GROUP_TEAM_ALL & !GROUP_NONE,
            role_membership: GROUP_STAGE,
            role_whitelist: GROUP_VOLUME | GROUP_GIANT_BOUNDING,
        };
    }

    #[inline]
    pub fn new_volume(team: CollisionTeam, filter: CollisionTeamFilter) -> CollisionGroups {
        let team_whitelist = match filter {
            CollisionTeamFilter::None => 0,
            CollisionTeamFilter::All => GROUP_TEAM_ALL,
            CollisionTeamFilter::Friend => team.friend(),
            CollisionTeamFilter::Enemy => team.enemy(),
        };
        return CollisionGroups {
            team_membership: team.friend(),
            team_whitelist,
            role_membership: GROUP_VOLUME,
            role_whitelist: GROUP_STAGE | GROUP_VOLUME | GROUP_GIANT_VOLUME,
        };
    }

    #[inline]
    pub fn new_giant_bounding(team: CollisionTeam) -> CollisionGroups {
        return CollisionGroups {
            team_membership: team.friend(),
            team_whitelist: GROUP_TEAM_ALL,
            role_membership: GROUP_GIANT_BOUNDING,
            role_whitelist: GROUP_STAGE | GROUP_GIANT_BOUNDING,
        };
    }

    #[inline]
    pub fn new_giant_volume(team: CollisionTeam) -> CollisionGroups {
        return CollisionGroups {
            team_membership: team.friend(),
            team_whitelist: GROUP_TEAM_ALL,
            role_membership: GROUP_GIANT_VOLUME,
            role_whitelist: GROUP_VOLUME,
        };
    }

    #[inline]
    pub fn new_target_box(team: CollisionTeam, shield: bool) -> CollisionGroups {
        let team_whitelist = match shield {
            true => team.enemy() & !GROUP_NONE,
            false => GROUP_TEAM_ALL & !GROUP_NONE,
        };
        return CollisionGroups {
            team_membership: team.friend(),
            team_whitelist,
            role_membership: GROUP_TARGET_BOX,
            role_whitelist: GROUP_HIT_BOX,
        };
    }

    #[inline]
    pub fn new_hit_box(
        team: CollisionTeam,
        filter: CollisionTeamFilter,
        stage: bool,
    ) -> CollisionGroups {
        let team_whitelist = match filter {
            CollisionTeamFilter::None => 0,
            CollisionTeamFilter::All => GROUP_TEAM_ALL,
            CollisionTeamFilter::Friend => team.friend(),
            CollisionTeamFilter::Enemy => team.enemy(),
        };
        let role_whitelist = match stage {
            true => GROUP_STAGE | GROUP_TARGET_BOX,
            false => GROUP_TARGET_BOX,
        };
        return CollisionGroups {
            team_membership: team.friend(),
            team_whitelist,
            role_membership: GROUP_HIT_BOX,
            role_whitelist,
        };
    }

    #[inline]
    pub fn can_interact_with(&self, other: &CollisionGroups) -> bool {
        return (self.team_membership & other.team_whitelist != 0)
            & (other.team_membership & self.team_whitelist != 0)
            & (self.role_membership & other.role_whitelist != 0)
            & (other.role_membership & self.role_whitelist != 0);
    }
}
