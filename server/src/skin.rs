use bytes::{BytesMut, Buf};
use common::{VResult, ReadExtensions};

/// Size of arms of a skin.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArmSize {
    /// Used for female characters.
    Slim,
    /// Used for male characters.
    Wide,
}

/// Type of a persona piece.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PersonaPieceType {
    Skeleton,
    Body,
    Skin,
    Bottom,
    Feet,
    Top,
    Mouth,
    Hair,
    Eyes,
    FacialHair,
}

/// Piece of a persona skin.
#[derive(Debug)]
pub struct PersonaPiece {
    /// UUID that identifies the piece itself.
    pub piece_id: String,
    /// Type of the persona piece.
    pub piece_type: PersonaPieceType,
    /// UUID that identifies the packet that this piece belongs to.
    pub pack_id: String,
    /// Whether this piece is from one of the default skins (Steve, Alex)
    pub default: bool,
    /// UUID that identifies a purchases persona piece.
    /// Empty for default pieces.
    pub product_id: String,
}

/// Colours for a persona piece.
#[derive(Debug)]
pub struct PersonaPieceTint {
    /// Persona piece type to tint.
    pub piece_type: PersonaPieceType,
    /// Colours that refer to different parts of the piece.
    pub colors: Vec<String>,
}

/// Animation type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SkinAnimationType {
    Head,
    Body32x32,
    Body128x128,
}

/// Expression type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SkinExpressionType {
    Linear,
    Blinking,
}

/// A skin animation.
#[derive(Debug)]
pub struct SkinAnimation {
    /// Width of the animation image in pixels.
    pub image_width: u32,
    /// Height of the animation image in pixels.
    pub image_height: u32,
    /// Image data.
    pub image_data: BytesMut,
    /// Animation type.
    pub animation_type: SkinAnimationType,
    /// Amount of frames in animation.
    pub frame_count: u32,
    /// Expression type.
    pub expression_type: SkinExpressionType,
}

/// A classic or persona skin.
#[derive(Debug)]
pub struct Skin {
    /// UUID created for the skin.
    pub skin_id: String,
    /// PlayFab ID created for the skin.
    /// PlayFab hosts the marketplace.
    pub playfab_id: String,
    /// Unknown what this does.
    pub resource_patch: BytesMut,
    /// Width of the skin image in pixels.
    pub image_width: u32,
    /// Height of the skin image in pixels.
    pub image_height: u32,
    /// Skin image data.
    pub image_data: BytesMut,
    /// Animations that the skin possesses.
    pub animations: Vec<SkinAnimation>,
    /// Width of the cape image in pixels.
    pub cape_image_width: u32,
    /// Height of the cape image in pixels.
    pub cape_image_height: u32,
    /// Cape image data
    pub cape_image_data: BytesMut,
    /// JSON containing information like bones.
    pub skin_geometry: BytesMut,
    /// Engine version for geometry data.
    pub geometry_data_engine_version: BytesMut,
    /// Whether this skin was purchased from the marketplace.
    pub premium_skin: bool,
    /// Whether this skin is a persona skin.
    pub persona_skin: bool,
    /// Whether the skin is classic but has a persona cape equipped.
    pub persona_cape_on_classic: bool,
    /// Unknown what this does.
    pub primary_user: bool,
    /// UUID that identifiers the skin's cape.
    pub cape_id: String,
    /// Skin colour.
    pub color: String,
    /// Size of the arms.
    pub arm_size: ArmSize,
    /// All persona pieces that consitute the skin.
    pub persona_pieces: Vec<PersonaPiece>,
    /// List of colours for the persona pieces.
    pub persona_piece_tints: Vec<PersonaPieceTint>,
}

fn get_bytes(buffer: &mut BytesMut) -> VResult<BytesMut> {
    let length = buffer.get_var_u32()?;
    let cursor = buffer.len() - buffer.remaining();

    let data = BytesMut::from(&buffer.as_ref()[cursor..cursor + length as usize]);
    buffer.advance(length as usize);

    Ok(data)
}

impl Skin {
    pub fn decode(buffer: &mut BytesMut) -> VResult<Self> {
        let skin_id = buffer.get_string()?;
        let playfab_id = buffer.get_string()?;
        let resource_patch = get_bytes(buffer)?;

        let resource_patch = {
            let length = buffer.get_var_u32()?;
            let cursor = buffer.len() - buffer.remaining();

            let data = BytesMut::from(&buffer.as_ref()[cursor..cursor + length as usize]);
            buffer.advance(length as usize);

            data
        };

        todo!()
    }
}