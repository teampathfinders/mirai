use bytes::{Buf, BytesMut, BufMut};
use common::{ReadExtensions, VResult, Encodable, WriteExtensions};

/// Size of arms of a skin.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArmSize {
    /// Used for female characters.
    Slim,
    /// Used for male characters.
    Wide,
}

impl ArmSize {
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Slim => "slim",
            Self::Wide => "wide"
        }
    } 
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

impl PersonaPieceType {
    #[inline]
    fn name(&self) -> &'static str {
        match self {
            Self::Skeleton => "persona_skeleton",
            Self::Body => "persona_body",
            Self::Skin => "persona_skin",
            Self::Bottom => "persona_bottom",
            Self::Feet => "persona_feet",
            Self::Top => "persona_top",
            Self::Mouth => "persona_mouth",
            Self::Hair => "persona_hair",
            Self::Eyes => "persona_eyes",
            Self::FacialHair => "persona_facial_hair"
        }
    }
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

impl PersonaPiece {
    fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.piece_id);
        buffer.put_string(self.piece_type.name());
        buffer.put_string(&self.pack_id);
        buffer.put_bool(self.default);
        buffer.put_string(&self.product_id);
    }
}

/// Colours for a persona piece.
#[derive(Debug)]
pub struct PersonaPieceTint {
    /// Persona piece type to tint.
    pub piece_type: PersonaPieceType,
    /// Colours that refer to different parts of the piece.
    pub colors: Vec<String>,
}

impl PersonaPieceTint {
    fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.piece_type.name());

        buffer.put_u32_le(self.colors.len() as u32);
        for color in &self.colors {
            buffer.put_string(color);    
        }
    }
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
    /// I have no idea why this is a float.
    pub frame_count: f32,
    /// Expression type.
    pub expression_type: SkinExpressionType,
}

impl SkinAnimation {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_u32_le(self.image_width);
        buffer.put_u32_le(self.image_height);
        
        buffer.put_var_u32(self.image_data.len() as u32);
        buffer.put(self.image_data.as_ref());

        buffer.put_u32_le(self.animation_type as u32);
        buffer.put_f32_le(self.frame_count);
        buffer.put_u32_le(self.expression_type as u32);
    }
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
    pub animation_data: BytesMut,
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
    /// UUID of the entire skin.
    pub full_id: String,
    /// Skin colour.
    pub color: String,
    /// Size of the arms.
    pub arm_size: ArmSize,
    /// All persona pieces that consitute the skin.
    pub persona_pieces: Vec<PersonaPiece>,
    /// List of colours for the persona pieces.
    pub persona_piece_tints: Vec<PersonaPieceTint>,
    /// Whether the skin is "trusted" by Minecraft.
    /// The server shouldn't actually trust this because the client can change it.
    pub trusted: bool,
}

fn get_bytes(buffer: &mut BytesMut) -> VResult<BytesMut> {
    let length = buffer.get_var_u32()?;
    let cursor = buffer.len() - buffer.remaining();

    let data =
        BytesMut::from(&buffer.as_ref()[cursor..cursor + length as usize]);
    buffer.advance(length as usize);

    Ok(data)
}

impl Skin {
    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.skin_id);
        buffer.put_string(&self.playfab_id);
        
        buffer.put_var_u32(self.resource_patch.len() as u32);
        buffer.put(self.resource_patch.as_ref());

        buffer.put_u32_le(self.image_width);
        buffer.put_u32_le(self.image_height);
        buffer.put_var_u32(self.image_data.len() as u32);
        buffer.put(self.image_data.as_ref());

        buffer.put_u32_le(self.animations.len() as u32);
        for animation in &self.animations {
            animation.encode(buffer);
        }

        buffer.put_u32_le(self.cape_image_width);
        buffer.put_u32_le(self.cape_image_height);
        buffer.put_var_u32(self.cape_image_data.len() as u32);
        buffer.put(self.cape_image_data.as_ref());

        buffer.put_var_u32(self.skin_geometry.len() as u32);
        buffer.put(self.skin_geometry.as_ref());

        buffer.put_var_u32(self.geometry_data_engine_version.len() as u32);
        buffer.put(self.geometry_data_engine_version.as_ref());

        buffer.put_var_u32(self.animation_data.len() as u32);
        buffer.put(self.animation_data.as_ref());

        buffer.put_string(&self.cape_id);
        buffer.put_string(&self.full_id);
        buffer.put_string(self.arm_size.name());
        buffer.put_string(&self.color);

        buffer.put_u32_le(self.persona_pieces.len() as u32);
        for piece in &self.persona_pieces {
            piece.encode(buffer);
        }

        buffer.put_u32_le(self.persona_piece_tints.len() as u32);
        for tint in &self.persona_piece_tints {
            tint.encode(buffer);
        }

        buffer.put_bool(self.premium_skin);
        buffer.put_bool(self.persona_skin);
        buffer.put_bool(self.persona_cape_on_classic);
        buffer.put_bool(self.primary_user);
    }
}
