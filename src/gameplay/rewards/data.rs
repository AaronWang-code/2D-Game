use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardType {
    RecoverHealth,
    EnhanceMeleeWeapon,
    IncreaseAttackSpeed,
    IncreaseAttackPower,
    IncreaseMaxHealth,
    ReduceDashCooldown,
    LifeStealOnKill,
    IncreaseCritChance,
    IncreaseMoveSpeed,
    DashDamageTrail,
    EnhanceRangedWeapon,
}
