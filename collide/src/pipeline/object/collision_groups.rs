pub struct CollisionGroups {
    pub team_membership: u16,
    pub team_whitelist: u16,
    pub role_membership: u16,
    pub role_whitelist: u16,
}

impl CollisionGroups {
    #[inline]
    pub fn can_interact_with(&self, other: &CollisionGroups) -> bool {
        return (self.team_membership & other.team_whitelist != 0)
            & (other.team_membership & self.team_whitelist != 0)
            & (self.role_membership & other.role_whitelist != 0)
            & (other.role_membership & self.role_whitelist != 0);
    }
}
