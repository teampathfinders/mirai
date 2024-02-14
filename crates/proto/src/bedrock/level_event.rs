use util::{bail, Deserialize, Serialize, Vector};
use util::{BinaryRead, BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// The type of level event that occurred.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LevelEventType {
    /// Plays a click sound.
    SoundClick = 1000,
    /// Plays a failed click sound.
    SoundClickFail = 1001,
    /// Plays the firework launch sound.
    SoundLaunch = 1002,
    /// Plays the sound of a door opening.
    SoundOpenDoor = 1003,
    SoundFizz = 1004,
    /// Plays the TNT fuse sound.
    SoundFuse = 1005,
    SoundPlayRecording = 1006,
    /// Plays the sound a ghast makes before it shoots a fireball.
    SoundGhastWarning = 1007,
    /// Plays the sound of a ghast fireball.
    SoundGhastFireball = 1008,
    /// Plays the sound of a blaze fireball.
    SoundBlazeFireball = 1009,
    /// Plays the sound of a zombie attempting to break a wooden door.
    SoundZombieWoodenDoor = 1010,
    /// Plays the sound of a zombie breaking a door.
    SoundZombieDoorCrash = 1012,
    /// Plays the sound of a villager being infected.
    SoundZombieInfected = 1016,
    /// Plays the sound of a zombie being converted into a drowned.
    SoundZombieConverted = 1017,
    /// Plays the sound of an enderman teleporting.
    SoundEndermanTeleport = 1018,
    /// Plays the sound of an anvil breaking.
    SoundAnvilBroken = 1020,
    /// Plays the sound of an anvil being used.
    SoundAnvilUsed = 1021,
    /// Plays the sound of a landing anvil.
    SoundAnvilLand = 1022,
    SoundInfinityArrowPickup = 1030,
    /// Plays the sound when an ender pearl teleports the player.
    SoundTeleportEnderPearl = 1032,
    SoundAddItem = 1040,
    /// Plays the sound of an item frame breaking.
    SoundItemFrameBreak = 1041,
    /// Plays the sound of an item being placed in an item frame.
    SoundItemFramePlace = 1042,
    /// Plays the sound of an item being removed from a frame.
    SoundItemFrameRemoveItem = 1043,
    /// Plays the sound of an item in an item frame being rotated.
    SoundItemFrameRotateItem = 1044,
    /// Plays the sound of a picked up experience orb.
    SoundExperienceOrbPickup = 1051,
    /// Plays the sound of a used totem of undying.
    SoundTotemUsed = 1052,
    /// Plays the sound of an armour stand breaking.
    SoundArmorStandBreak = 1060,
    /// Plays the sound of an armour stand being hit.
    SoundArmorStandHit = 1061,
    /// Plays the sound of an armour stand landing.
    SoundArmorStandLand = 1062,
    /// Plays the sound of an armour stand being placed.
    SoundArmorStandPlace = 1063,
    /// Plays the sound of pointed dripstone landing.
    SoundPointedDripstoneLand = 1064,
    /// Plays the sound of a dye being used.
    SoundDyeUsed = 1065,
    /// Plays the sound of an ink sac being used.
    SoundInkSacUsed = 1066,
    /// Queues a custom song.
    QueueCustomMusic = 1900,
    /// Plays a custom song.
    PlayCustomMusic = 1901,
    /// Stops a custom song.
    StopCustomMusic = 1902,
    /// Sets the volume of the music.
    SetMusicVolume = 1903,
    ParticlesShoot = 2000,
    ParticlesDestroyBlock = 2001,
    /// Spawns potion splash particles.
    ParticlesPotionSplash = 2002,
    ParticlesEyeOfEnderDeath = 2003,
    ParticlesMobBlockSpawn = 2004,
    /// Spawns particles when a crop grows.
    ParticlesCropGrowth = 2005,
    /// Plays the guardian ghost effect.
    ParticlesSoundGuardianGhost = 2006,
    /// Spawns the particles of a mob dying.
    ParticlesDeathSmoke = 2007,
    /// Spawns the particles of a deny block.
    ParticlesDenyBlock = 2008,
    ParticlesGenericSpawn = 2009,
    ParticlesDragonEgg = 2010,
    ParticlesCropEaten = 2011,
    /// Spawns critical hit particles.
    ParticlesCritical = 2012,
    ParticlesTeleport = 2013,
    ParticlesCrackBlock = 2014,
    ParticlesBubble = 2015,
    ParticlesEvaporate = 2016,
    ParticlesDestroyArmorStand = 2017,
    ParticlesBreakingEgg = 2018,
    ParticlesDestroyEgg = 2019,
    ParticlesEvaporateWater = 2020,
    ParticlesDestroyBlockNoSound = 2021,
    ParticlesKnockbackRoar = 2022,
    ParticlesTeleportTrail = 2023,
    ParticlesPointCloud = 2024,
    /// Spawns explosion particles.
    ParticlesExplosion = 2025,
    ParticlesBlockExplosion = 2026,
    ParticlesVibrationSignal = 2027,
    /// Spawns dripstone dripping particles.
    ParticlesDripstoneDrip = 2028,
    ParticlesFizzEffect = 2029,
    /// Adds wax to a copper block.
    WaxOn = 2030,
    /// Removes the wax from a copper block.
    WaxOff = 2031,
    /// Scrapes the oxidisation of a copper block.
    Scrape = 2032,
    /// Spawns an electric spark particle.
    ParticlesElectricSpark = 2033,
    /// Spawns turtle egg particles.
    ParticlesTurtleEgg = 2034,
    /// Spawns sculk shriek particles.
    ParticlesSculkShriek = 2035,
    SculkCatalystBloom = 2036,
    SculkCharge = 2037,
    SculkChargePop = 2038,
    /// Spawns the Warden sonic explosion attack.
    SonicExplosion = 2039,
    /// It has started raining.
    StartRaining = 3001,
    /// A thunderstorm has started.
    StartThunderstorm = 3002,
    /// It has stopped raining.
    StopRaining = 3003,
    /// It has stopped thundering.
    StopThunderstorm = 3004,
    /// The game has been paused.
    GlobalPause = 3005,
    SimTimeStep = 3006,
    SimTimeScale = 3007,
    ActivateBlock = 3500,
    CauldronExplode = 3501,
    /// Armor was dyed in a cauldron.
    CauldronDyeArmor = 3502,
    CauldronCleanArmor = 3503,
    /// A potion has been emptied in the cauldron.
    CauldronFillPotion = 3504,
    /// A potion has been taken from the cauldron.
    CauldronTakePotion = 3505,
    /// The cauldron has been filled with water.
    CauldronFillWater = 3506,
    /// All water has been taken out of the cauldron.
    CauldronTakeWater = 3507,
    /// A dye has been added to the cauldron.
    CauldronAddDye = 3508,
    /// A banner has been cleaned in the cauldron.
    CauldronCleanBanner = 3509,
    /// The cauldron has been flushed.
    CauldronFlush = 3510,
    /// A code agent has been spawned.
    AgentSpawnEffect = 3511,
    /// A cauldron has been filled with lava.
    CauldronFillLava = 3512,
    /// Lava has been taken out of the cauldron.
    CauldronTakeLava = 3513,
    /// A cauldron has been filled with powder snow.
    CauldronFillPowderSnow = 3514,
    /// Powder snow has been taken out of the cauldron.
    CauldronTakePowderSnow = 3515,
    /// A block has started cracking.
    StartBlockCracking = 3600,
    /// A block has stopped cracking.
    StopBlockCracking = 3601,
    /// Updates the state of block cracking.
    UpdateBlockCracking = 3602,
    /// All players are sleeping.
    AllPlayersSleeping = 9800,
    /// Some players have started sleeping.
    SleepingPlayers = 9801,
    JumpPrevented = 9810,
    ParticlesLegacyEvent = 0x4000,
}

impl TryFrom<i32> for LevelEventType {
    type Error = anyhow::Error;

    #[allow(clippy::too_many_lines)] // Splitting this will be a little difficult...
    fn try_from(value: i32) -> anyhow::Result<Self> {
        Ok(match value {
            1000 => Self::SoundClick,
            1001 => Self::SoundClickFail,
            1002 => Self::SoundLaunch,
            1003 => Self::SoundOpenDoor,
            1004 => Self::SoundFizz,
            1005 => Self::SoundFuse,
            1006 => Self::SoundPlayRecording,
            1007 => Self::SoundGhastWarning,
            1008 => Self::SoundGhastFireball,
            1009 => Self::SoundBlazeFireball,
            1010 => Self::SoundZombieWoodenDoor,
            1012 => Self::SoundZombieDoorCrash,
            1016 => Self::SoundZombieInfected,
            1017 => Self::SoundZombieConverted,
            1018 => Self::SoundEndermanTeleport,
            1020 => Self::SoundAnvilBroken,
            1021 => Self::SoundAnvilUsed,
            1022 => Self::SoundAnvilLand,
            1030 => Self::SoundInfinityArrowPickup,
            1032 => Self::SoundTeleportEnderPearl,
            1040 => Self::SoundAddItem,
            1041 => Self::SoundItemFrameBreak,
            1042 => Self::SoundItemFramePlace,
            1043 => Self::SoundItemFrameRemoveItem,
            1044 => Self::SoundItemFrameRotateItem,
            1051 => Self::SoundExperienceOrbPickup,
            1052 => Self::SoundTotemUsed,
            1060 => Self::SoundArmorStandBreak,
            1061 => Self::SoundArmorStandHit,
            1062 => Self::SoundArmorStandLand,
            1063 => Self::SoundArmorStandPlace,
            1064 => Self::SoundPointedDripstoneLand,
            1065 => Self::SoundDyeUsed,
            1066 => Self::SoundInkSacUsed,
            1900 => Self::QueueCustomMusic,
            1901 => Self::PlayCustomMusic,
            1902 => Self::StopCustomMusic,
            1903 => Self::SetMusicVolume,
            2000 => Self::ParticlesShoot,
            2001 => Self::ParticlesDestroyBlock,
            2002 => Self::ParticlesPotionSplash,
            2003 => Self::ParticlesEyeOfEnderDeath,
            2004 => Self::ParticlesMobBlockSpawn,
            2005 => Self::ParticlesCropGrowth,
            2006 => Self::ParticlesSoundGuardianGhost,
            2007 => Self::ParticlesDeathSmoke,
            2008 => Self::ParticlesDenyBlock,
            2009 => Self::ParticlesGenericSpawn,
            2010 => Self::ParticlesDragonEgg,
            2011 => Self::ParticlesCropEaten,
            2012 => Self::ParticlesCritical,
            2013 => Self::ParticlesTeleport,
            2014 => Self::ParticlesCrackBlock,
            2015 => Self::ParticlesBubble,
            2016 => Self::ParticlesEvaporate,
            2017 => Self::ParticlesDestroyArmorStand,
            2018 => Self::ParticlesBreakingEgg,
            2019 => Self::ParticlesDestroyEgg,
            2020 => Self::ParticlesEvaporateWater,
            2021 => Self::ParticlesDestroyBlockNoSound,
            2022 => Self::ParticlesKnockbackRoar,
            2023 => Self::ParticlesTeleportTrail,
            2024 => Self::ParticlesPointCloud,
            2025 => Self::ParticlesExplosion,
            2026 => Self::ParticlesBlockExplosion,
            2027 => Self::ParticlesVibrationSignal,
            2028 => Self::ParticlesDripstoneDrip,
            2029 => Self::ParticlesFizzEffect,
            2030 => Self::WaxOn,
            2031 => Self::WaxOff,
            2032 => Self::Scrape,
            2033 => Self::ParticlesElectricSpark,
            2034 => Self::ParticlesTurtleEgg,
            2035 => Self::ParticlesSculkShriek,
            2036 => Self::SculkCatalystBloom,
            2037 => Self::SculkCharge,
            2038 => Self::SculkChargePop,
            2039 => Self::SonicExplosion,
            3001 => Self::StartRaining,
            3002 => Self::StartThunderstorm,
            3003 => Self::StopRaining,
            3004 => Self::StopThunderstorm,
            3005 => Self::GlobalPause,
            3006 => Self::SimTimeStep,
            3007 => Self::SimTimeScale,
            3500 => Self::ActivateBlock,
            3501 => Self::CauldronExplode,
            3502 => Self::CauldronDyeArmor,
            3503 => Self::CauldronCleanArmor,
            3504 => Self::CauldronFillPotion,
            3505 => Self::CauldronTakePotion,
            3506 => Self::CauldronFillWater,
            3507 => Self::CauldronTakeWater,
            3508 => Self::CauldronAddDye,
            3509 => Self::CauldronCleanBanner,
            3510 => Self::CauldronFlush,
            3511 => Self::AgentSpawnEffect,
            3512 => Self::CauldronFillLava,
            3513 => Self::CauldronTakeLava,
            3514 => Self::CauldronFillPowderSnow,
            3515 => Self::CauldronTakePowderSnow,
            3600 => Self::StartBlockCracking,
            3601 => Self::StopBlockCracking,
            3602 => Self::UpdateBlockCracking,
            9800 => Self::AllPlayersSleeping,
            9801 => Self::SleepingPlayers,
            9810 => Self::JumpPrevented,
            0x4000 => Self::ParticlesLegacyEvent,
            _ => bail!(Malformed, "Invalid level event type {value}")
        })
    }
}

/// A level event.
#[derive(Debug, Clone)]
pub struct LevelEvent {
    /// Type of level event that occurred.
    pub event_type: LevelEventType,
    /// Position where the event occurred.
    pub position: Vector<f32, 3>,
    /// Data associated with the event.
    pub event_data: i32,
}

impl ConnectedPacket for LevelEvent {
    const ID: u32 = 0x19;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.event_type as i32) + 3 * 4 +
            size_of_varint(self.event_data)
    }
}

impl Serialize for LevelEvent {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i32(self.event_type as i32)?;
        writer.write_vecf(&self.position)?;
        writer.write_var_i32(self.event_data)
    }
}

impl<'a> Deserialize<'a> for LevelEvent {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let event_type = LevelEventType::try_from(reader.read_var_i32()?)?;
        let position = reader.read_vecf()?;
        let event_data = reader.read_var_i32()?;

        Ok(Self {
            event_type,
            position,
            event_data,
        })
    }
}