/// Wrapper around the different types of gamerule value types
/// to be able to store them in a single map.
#[derive(Copy, Clone)]
pub enum RuleValue {
    /// A boolean value.
    Bool(bool),
    /// An integer value.
    I32(i32)
}

impl From<bool> for RuleValue {
    #[inline]
    fn from(value: bool) -> RuleValue { RuleValue::Bool(value) }
}

impl From<i32> for RuleValue {
    #[inline]
    fn from(value: i32) -> RuleValue { RuleValue::I32(value) }
}

impl From<RuleValue> for bool {
    fn from(value: RuleValue) -> bool {
        match value {
            RuleValue::Bool(value) => value,
            _ => unreachable!("Gamerule value was of wrong type, expected bool, got i32")
        }
    }
}

impl From<RuleValue> for i32 {
    fn from(value: RuleValue) -> i32 {
        match value {
            RuleValue::I32(value) => value,
            _ => unreachable!("Gamerule value was of wrong type, expected i32, got bool")
        }
    }
}

/// Creates a new gamerule with optional default value and stringified name.
#[macro_export]
macro_rules! gamerule {
    ($name: ident: $ty: ty = $default: expr, $str_name: expr) => {
        paste::paste! {
            #[doc = r"The `" $name "` gamerule"]
            #[doc = r"This gamerule is of type `" $ty "`, has a default value of `" $default "` and is referred to as `" $str_name "` in commands."]
            pub enum $name {}

            impl Rule for $name {
                type Value = $ty;

                const NAME: &'static str = $str_name;
                const IS_VANILLA: bool = false;
                
                #[inline]
                fn default() -> Self::Value {
                    $default
                }
            }
        }
    };
    ($name: ident: $ty: ty = $default: expr) => {
        paste::paste! {
            gamerule!($name: $ty = $default, stringify!([<$name:lower>]));
        }
    };
    ($name: ident: $ty: ty) => {
        gamerule!($name: $ty = <$ty>::default());
    }
}

pub use gamerule;

/// Implemented by all gamerules.
/// 
/// All vanilla gamerules have already been implemented but you can also implement your own custom gamerules.
/// ```ignore
/// gamerule!(MyNamedGameRule: i32 = 42, "customgamerulename");
/// gamerule!(MyDefaultGamerule: i32 = 12);
/// gamerule!(MyGamerule: i32);
/// ```
pub trait Rule: 'static {
    /// The inner value of this gamerule. This can either be `bool` or `i32`.
    type Value: From<RuleValue> + Default;
    /// The in-game name of this gamerule.
    const NAME: &'static str;
    /// Whether this gamerule is part of vanilla Minecraft.
    /// This should always be set to `false` for any user-defined gamerules.
    const IS_VANILLA: bool;
    /// Returns the default value of this gamerule.
    fn default() -> Self::Value;
}

/// Implements the internal gamerules.
macro_rules! impl_gamerules {
    ($($name: ident: $ty: ident = $default: literal - $str_name: literal),+) => {
        paste::paste! {
            $(
                #[doc = "The vanilla `" $name "` gamerule"]
                pub enum $name {}

                impl Rule for $name {
                    type Value = $ty;

                    const NAME: &'static str = $str_name;
                    const IS_VANILLA: bool = true;

                    #[inline]
                    fn default() -> Self::Value { $default }
                }
            )+
        }
    }
}

impl_gamerules!(
    CommandBlocksEnabled: bool = true - "commandblocksenabled",
    CommandBlockOutput: bool = true - "commandblockoutput",
    DaylightCycle: bool = true - "dodaylightcycle",
    EntityDrops: bool = true - "doentitydrops",
    FireTick: bool = true - "dofiretick",
    Insomnia: bool = true - "doinsomnia",
    ImmediateRespawn: bool = false - "doimmediaterespawn",
    LimitedCrafting: bool = false - "dolimitedcrafting",
    MobLoot: bool = true - "domobloot",
    MobSpawning: bool = true - "domobspawning",
    TileDrops: bool = true - "dotiledrops",
    WeatherCycle: bool = true - "doweathercycle",
    DrowningDamage: bool = true - "drowningdamage",
    FallDamage: bool = true - "falldamage",
    FireDamage: bool = true - "firedamage",
    FreezeDamage: bool = true - "freezedamage",
    FunctionCommandLimit: i32 = 10_000 - "functioncommandlimit",
    KeepInventory: bool = false - "keepinventory",
    MaxCommandChainLength: i32 = 65_536 - "maxcommandchainlength",
    MobGriefing: bool = true - "mobgriefing",
    NaturalRegeneration: bool = true - "naturalregeneration",
    PlayersSleepingPercentage: i32 = 100 - "playerssleepingpercentage",
    Pvp: bool = true - "pvp",
    RandomTickSpeed: i32 = 1 - "randomtickspeed",
    RecipesUnlock: bool = true - "recipesunlock",
    RespawnBlocksExplode: bool = true - "respawnblocksexplode",
    SendCommandFeedback: bool = true - "sendcommandfeedback",
    ShowBorderEffect: bool = true - "showbordereffect",
    ShowCoordinates: bool = true - "showcoordinates",
    ShowDeathMessages: bool = true - "showdeathmessages",
    ShowTags: bool = true - "showtags",
    SpawnRadius: i32 = 10 - "spawnradius",
    TntExplodes: bool = true - "tntexplodes"
);