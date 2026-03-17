use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardType {
    IncreaseAttackSpeed,
    IncreaseMaxHealth,
    ReduceDashCooldown,
    LifeStealOnKill,
    IncreaseCritChance,
    IncreaseMoveSpeed,
    DashDamageTrail,
    BonusProjectile,
}

