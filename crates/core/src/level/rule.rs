use macros::variant_count;

mod private {
   pub trait Sealed {}
   impl Sealed for bool {}
   impl Sealed for i32 {}
}

#[derive(Copy, Clone)]
pub enum GameRuleValue {
    Bool(bool),
    I32(i32)
}

pub trait ValidGameRuleValue: private::Sealed + Default {}

impl ValidGameRuleValue for bool {}
impl ValidGameRuleValue for i32 {}

/// Creates a new gamerule with optional default value and stringified name.
#[macro_export]
macro_rules! gamerule {
    ($name: ident: $ty: ty = $default: expr, $str_name: expr) => {
        pub enum $name {}

        paste::paste! {
            impl GameRule for $name {
                type Value = $ty;

                const NAME: &'static str = $str_name;
                
                #[inline]
                fn default() -> Self::Value {
                    $default
                }

                #[inline]
                fn from_value(value: GameRuleValue) -> Self::Value {
                    match value {
                        GameRuleValue::[<$ty:camel>](value) => value,
                        _ => unreachable!()
                        // _ => unreachable!(
                        //     "Gamerule ", stringify!($name), " expects a value of type " $ty
                        // )
                    }
                }

                #[inline]
                fn to_value(value: Self::Value) -> GameRuleValue {
                    GameRuleValue::[<$ty:camel>](value)
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
pub trait GameRule: 'static {
    /// The inner value of this gamerule. This can either be `bool` or `i32`.
    type Value: ValidGameRuleValue;

    /// The in-game name of this gamerule.
    const NAME: &'static str;

    /// Returns the default value of this gamerule.
    fn default() -> Self::Value;

    /// Converts a [`GameRuleValue``] enum to its inner value.
    fn from_value(value: GameRuleValue) -> Self::Value;

    /// Converts a value to a [`GameRuleValue`].
    fn to_value(value: Self::Value) -> GameRuleValue;
}

macro_rules! impl_gamerules {
    ($($name: ident: $ty: ident = $default: expr),+) => {
        paste::paste! {
            $(
                #[doc = "The `" $name "` gamerule"]
                pub enum $name {}

                impl GameRule for $name {
                    type Value = $ty;

                    const NAME: &'static str = stringify!([< $name:lower >]);

                    #[inline]
                    fn default() -> Self::Value { $default }

                    #[inline]
                    fn from_value(value: GameRuleValue) -> $ty {
                        match value {
                            GameRuleValue::[< $ty:camel >](value) => value,
                            _ => unreachable!("Gamerule has wrong value type")
                        }
                    }

                    #[inline]
                    fn to_value(value: $ty) -> GameRuleValue {
                        GameRuleValue::[< $ty:camel >](value)
                    }
                }
            )+
        }
    }
}

impl_gamerules!(
    CommandBlocksEnabled: bool = true,
    CommandBlockOutput: bool = true,
    DoDaylightCycle: bool = true,
    DoEntityDrops: bool = true,
    DoFireTick: bool = true,
    DoInsomnia: bool = true,
    DoImmediateRespawn: bool = false,
    DoLimitedCrafting: bool = false,
    DoMobLoot: bool = true,
    DoMobSpawning: bool = true,
    DoTileDrops: bool = true,
    DoWeatherCycle: bool = true,
    DrowningDamage: bool = true,
    FallDamage: bool = true,
    FireDamage: bool = true,
    FreezeDamage: bool = true,
    FunctionCommandLimit: i32 = 10_000,
    KeepInventory: bool = false,
    MaxCommandChainLength: i32 = 65_536,
    MobGriefing: bool = true,
    NaturalRegeneration: bool = true,
    PlayersSleepingPercentage: i32 = 100,
    Pvp: bool = true,
    RandomTickSpeed: i32 = 1,
    RecipesUnlock: bool = true,
    RespawnBlocksExplode: bool = true,
    SendCommandFeedback: bool = true,
    ShowBorderEffect: bool = true,
    ShowCoordinates: bool = true,
    ShowDeathMessages: bool = true,
    ShowTags: bool = true,
    SpawnRadius: i32 = 10,
    TntExplodes: bool = true
);

// #[derive(PartialEq, Eq, Hash)]
// pub enum GameRules {
//     ShowCoordinates
// }

// pub enum ShowCoordinates {}

// impl GameRule for ShowCoordinates {
//     type Value = bool;

//     const NAME: &'static str = "showcoordinates";
//     const VARIANT: GameRules = GameRules::ShowCoordinates;

//     #[inline]
//     fn from_value(value: GameRuleValue) -> bool { 
//         match value { 
//             GameRuleValue::Bool(value) => value,
//             _ => unreachable!()
//         } 
//     }
    
//     #[inline]
//     fn to_value(value: bool) -> GameRuleValue { GameRuleValue::Bool(value) }
// }