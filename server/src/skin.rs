use bytes::{Buf, BufMut, BytesMut};
use common::{
    bail, Encodable, ReadExtensions, VError, VResult, WriteExtensions,
};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

/// Size of arms of a skin.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
pub enum ArmSize {
    /// Used for female characters.
    #[serde(rename = "slim")]
    Slim,
    /// Used for male characters.
    #[serde(rename = "wide")]
    Wide,
}

impl ArmSize {
    #[inline]
    const fn name(&self) -> &'static str {
        match self {
            Self::Slim => "slim",
            Self::Wide => "wide",
        }
    }
}

/// Type of a persona piece.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
pub enum PersonaPieceType {
    #[serde(rename = "persona_skeleton")]
    Skeleton,
    #[serde(rename = "persona_body")]
    Body,
    #[serde(rename = "persona_skin")]
    Skin,
    #[serde(rename = "persona_bottom")]
    Bottom,
    #[serde(rename = "persona_feet")]
    Feet,
    #[serde(rename = "persona_top")]
    Top,
    #[serde(rename = "persona_mouth")]
    Mouth,
    #[serde(rename = "persona_hair")]
    Hair,
    #[serde(rename = "persona_eyes")]
    Eyes,
    #[serde(rename = "persona_facial_hair")]
    FacialHair,
    #[serde(rename = "persona_dress")]
    Dress,
}

impl PersonaPieceType {
    #[inline]
    const fn name(&self) -> &'static str {
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
            Self::FacialHair => "persona_facial_hair",
            Self::Dress => "persona_dress",
        }
    }
}

/// Piece of a persona skin.
#[derive(Debug, Deserialize, Clone)]
pub struct PersonaPiece {
    /// UUID that identifies the piece itself.
    #[serde(rename = "PieceId")]
    pub piece_id: String,
    /// Type of the persona piece.
    #[serde(rename = "PieceType")]
    pub piece_type: PersonaPieceType,
    /// UUID that identifies the packet that this piece belongs to.
    #[serde(rename = "PackId")]
    pub pack_id: String,
    /// Whether this piece is from one of the default skins (Steve, Alex)
    #[serde(rename = "IsDefault")]
    pub default: bool,
    /// UUID that identifies a purchases persona piece.
    /// Empty for default pieces.
    #[serde(rename = "ProductId")]
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
#[derive(Debug, Deserialize, Clone)]
pub struct PersonaPieceTint {
    /// Persona piece type to tint.
    #[serde(rename = "PieceType")]
    pub piece_type: PersonaPieceType,
    /// Colours that refer to different parts of the piece.
    #[serde(rename = "Colors")]
    pub colors: [String; 4],
}

impl PersonaPieceTint {
    fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(self.piece_type.name());

        buffer.put_u32_le(self.colors.len() as u32);
        for color in &self.colors {
            buffer.put_string(color);
        }
    }
}

/// Animation type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum SkinAnimationType {
    None,
    Head,
    Body32x32,
    Body128x128,
}

/// Expression type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum SkinExpressionType {
    Linear,
    Blinking,
}

/// A skin animation.
#[derive(Debug, Deserialize, Clone)]
pub struct SkinAnimation {
    /// Width of the animation image in pixels.
    #[serde(rename = "ImageWidth")]
    pub image_width: u32,
    /// Height of the animation image in pixels.
    #[serde(rename = "ImageHeight")]
    pub image_height: u32,
    /// Image data.
    #[serde(rename = "Image", with = "base64")]
    pub image_data: BytesMut,
    /// Animation type.
    #[serde(rename = "Type")]
    pub animation_type: SkinAnimationType,
    /// Amount of frames in animation.
    /// I have no idea why this is a float.
    #[serde(rename = "Frames")]
    pub frame_count: f32,
    /// Expression type.
    #[serde(rename = "AnimationExpression")]
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
#[derive(Debug, Deserialize, Clone)]
pub struct Skin {
    /// UUID created for the skin.
    #[serde(rename = "SkinId")]
    pub skin_id: String,
    /// PlayFab ID created for the skin.
    /// PlayFab hosts the marketplace.
    #[serde(rename = "PlayFabId")]
    pub playfab_id: String,
    /// Unknown what this does.
    #[serde(rename = "SkinResourcePatch", with = "base64_string")]
    pub resource_patch: String,
    /// Width of the skin image in pixels.
    #[serde(rename = "SkinImageWidth")]
    pub image_width: u32,
    /// Height of the skin image in pixels.
    #[serde(rename = "SkinImageHeight")]
    pub image_height: u32,
    /// Skin image data.
    #[serde(rename = "SkinData", with = "base64")]
    pub image_data: BytesMut,
    /// Animations that the skin possesses.
    #[serde(rename = "AnimatedImageData")]
    pub animations: Vec<SkinAnimation>,
    /// Width of the cape image in pixels.
    #[serde(rename = "CapeImageWidth")]
    pub cape_image_width: u32,
    /// Height of the cape image in pixels.
    #[serde(rename = "CapeImageHeight")]
    pub cape_image_height: u32,
    /// Cape image data
    #[serde(rename = "CapeData", with = "base64")]
    pub cape_image_data: BytesMut,
    /// JSON containing information like bones.
    #[serde(rename = "SkinGeometryData", with = "base64_string")]
    pub skin_geometry: String,
    #[serde(rename = "SkinAnimationData", with = "base64_string")]
    pub animation_data: String,
    /// Engine version for geometry data.
    #[serde(rename = "SkinGeometryDataEngineVersion", with = "base64_string")]
    pub geometry_data_engine_version: String,
    /// Whether this skin was purchased from the marketplace.
    #[serde(rename = "PremiumSkin")]
    pub premium_skin: bool,
    /// Whether this skin is a persona skin.
    #[serde(rename = "PersonaSkin")]
    pub persona_skin: bool,
    /// Whether the skin is classic but has a persona cape equipped.
    #[serde(rename = "CapeOnClassicSkin")]
    pub persona_cape_on_classic: bool,
    /// UUID that identifiers the skin's cape.
    #[serde(rename = "CapeId")]
    pub cape_id: String,
    /// Skin colour.
    #[serde(rename = "SkinColor")]
    pub color: String,
    /// Size of the arms.
    #[serde(rename = "ArmSize")]
    pub arm_size: ArmSize,
    /// All persona pieces that consitute the skin.
    #[serde(rename = "PersonaPieces")]
    pub persona_pieces: Vec<PersonaPiece>,
    /// List of colours for the persona pieces.
    #[serde(rename = "PieceTintColors")]
    pub persona_piece_tints: Vec<PersonaPieceTint>,
    /// Whether the skin is "trusted" by Minecraft.
    /// The server shouldn't actually trust this because the client can change it.
    #[serde(rename = "TrustedSkin")]
    pub trusted: bool,
}

/// Serde deserializer for raw base64.
mod base64 {
    use base64::Engine;
    use bytes::BytesMut;
    use serde::{Deserializer, Deserialize};

    const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

    // TODO: This can probably be done more efficiently by directly writing into the BytesMut buffer.
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BytesMut, D::Error> {
        let base64 = String::deserialize(d)?;

        let bytes = ENGINE.decode(base64).map_err(|e| serde::de::Error::custom(e))?;
        Ok(BytesMut::from(bytes.as_slice()))
    }
}

/// Serde deserializer that decodes the base64 and converts it into a string.
mod base64_string {
    use base64::Engine;
    use bytes::BytesMut;
    use serde::{Deserializer, Deserialize};

    const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        let base64 = String::deserialize(d)?;
        let bytes = ENGINE.decode(base64).map_err(|e| serde::de::Error::custom(e))?;

        String::from_utf8(bytes).map_err(|e| serde::de::Error::custom(e))
    }
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
    /// Validates skin dimensions.
    pub fn validate(&self) -> VResult<()> {
        if self.image_width * self.image_height * 4
            != self.image_data.len() as u32
        {
            bail!(
                InvalidSkin,
                "Invalid skin dimensions, image data does not match"
            )
        }

        if self.cape_image_width * self.cape_image_height * 4
            != self.cape_image_data.len() as u32
        {
            bail!(
                InvalidSkin,
                "Invalid cape dimensions, image data does not match"
            )
        }

        for animation in &self.animations {
            if animation.image_width * animation.image_height * 4
                != animation.image_data.len() as u32
            {
                bail!(
                    InvalidSkin,
                    "Invalid animation dimensions, image data does not match"
                )
            }
        }

        Ok(())
    }

    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_string(&self.skin_id);
        buffer.put_string(&self.playfab_id);
        buffer.put_string(&self.resource_patch);

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

        buffer.put_string(&self.skin_geometry);
        buffer.put_string(&self.geometry_data_engine_version);
        buffer.put_string(&self.animation_data);

        buffer.put_string(&self.cape_id);
        buffer.put_string(&self.skin_id); // Full ID
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
        buffer.put_bool(true); // Primary user.
    }
}
