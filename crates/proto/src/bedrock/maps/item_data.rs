use bitflags::bitflags;

use util::{BinaryWrite, BlockPosition, Serialize, Vector};

use crate::{bedrock::ConnectedPacket, types::Dimension};

use super::Rgba;

/// Type of decoration marker to put on the map.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MapDecorationStyle {
    MarkerWhite = 0,
    MarkerGreen,
    MarkerRed,
    MarkerBlue,
    CrossWhite,
    TriangleRed,
    SquareWhite,
    MarkerSign,
    MarkerPink,
    MarkerOrange,
    MarkerYellow,
    MarkerTeal,
    TriangleGreen,
    SmallSquareWhite,
    Mansion,
    Monument,
    NoDraw,
    VillageDesert,
    VillagePlains,
    VillageSavanna,
    VillageSnowy,
    VillageTaiga,
    JungleTemple,
    WitchHut
}

/// Describes a decoration item added to the map.
/// These are markers that can be set at certain locations with
/// an optional styling.
#[derive(Debug, Clone)]
pub struct MapDecoration<'a> {
    /// Decoration style.
    pub style: MapDecorationStyle,
    /// One of the 16 possible rotations of the marker.
    pub rotation: u8,
    /// X offset in pixels.
    pub x: u8,
    /// Y offset in pixels.
    pub y: u8,
    /// Text label added to the marker.
    pub label: &'a str,
    /// Optional colour of the marker.
    /// Some marker styles already have a predefined colour however.
    pub color: Rgba
}

impl<'a> Serialize for MapDecoration<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(self.style as u8)?;
        writer.write_u8(self.rotation)?;
        writer.write_u8(self.x)?;
        writer.write_u8(self.y)?;
        writer.write_str(self.label)?;
        
        // Why are they using variable encoding here rather than bytes?
        writer.write_var_u32(self.color.x as u32)?;
        writer.write_var_u32(self.color.y as u32)?;
        writer.write_var_u32(self.color.z as u32)?;
        writer.write_var_u32(self.color.w as u32)   
    }
}

#[derive(Debug, Clone)]
#[repr(i32)]
pub enum TrackedMapObjectType {
    Entity,
    Block
}

/// Describes an object that is being tracked by the map.
/// This can be a (moving) entity or a block at a certain position.
#[derive(Debug, Clone)]
pub enum TrackedMapObject {
    Entity { unique_id: i64 },
    Block { position: BlockPosition }
}

impl Serialize for TrackedMapObject {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            TrackedMapObject::Entity { unique_id } => {
                writer.write_i32_le(TrackedMapObjectType::Entity as i32)?;
                writer.write_var_i64(*unique_id)?;
            },
            TrackedMapObject::Block { position } => {
                writer.write_i32_le(TrackedMapObjectType::Block as i32)?;
                writer.write_block_pos(position)?;
            }
        }

        Ok(())
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct MapUpdateFlags: u32 {
        const TEXTURE = 1 << 1;
        const DECORATION = 1 << 2;
        const INITIALIZATION = 1 << 3;
    }
}

#[derive(Debug, Clone)]
pub struct MapInitializationUpdate<'a> {
    included_in: &'a [i64],
}

#[derive(Debug, Clone)]
pub struct MapDecorationUpdate<'a> {
    trackers: &'a [TrackedMapObject],
    decorations: &'a [MapDecoration<'a>]
}

#[derive(Debug, Clone)]
pub struct MapTextureUpdate<'a> {
    width: i32,
    height: i32,
    xoffset: i32,
    yoffset: i32,
    pixels: &'a [Rgba]
}

#[derive(Debug, Clone)]
pub struct MapItemData<'a> {
    pub map_id: i64,
    pub dimension: Dimension,
    pub locked: bool,
    pub origin: BlockPosition,
    pub scale: u8,
    pub initialization: Option<MapInitializationUpdate<'a>>,
    pub decoration: Option<MapDecorationUpdate<'a>>,
    pub texture: Option<MapTextureUpdate<'a>>
}

impl<'a> ConnectedPacket for MapItemData<'a> {
    const ID: u32 = 43;
}

impl<'a> Serialize for MapItemData<'a> {
    fn serialize_into<W: BinaryWrite>(&self, mut writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_i64(self.map_id)?;

        let mut update_flags = MapUpdateFlags::empty();
        if self.initialization.is_some() {
            update_flags |= MapUpdateFlags::INITIALIZATION;
        }

        if self.decoration.is_some() {
            update_flags |= MapUpdateFlags::DECORATION;
        }

        if self.texture.is_some() {
            update_flags |= MapUpdateFlags::TEXTURE;
        }

        writer.write_var_u32(update_flags.bits())?;
        writer.write_u8(self.dimension as u8)?;
        writer.write_bool(self.locked)?;
        writer.write_block_pos(&self.origin)?;

        if let Some(update) = &self.initialization {
            writer.write_var_u32(update.included_in.len() as u32)?;
            for map in update.included_in {
                writer.write_var_i64(*map)?;
            }
        }

        writer.write_u8(self.scale)?;

        if let Some(update) = &self.decoration {
            writer.write_var_u32(update.trackers.len() as u32)?;
            for tracker in update.trackers {
                tracker.serialize_into(&mut writer)?;
            }

            writer.write_var_u32(update.decorations.len() as u32)?;
            for deco in update.decorations {
                deco.serialize_into(&mut writer)?;
            }
        }

        Ok(())
    }
}