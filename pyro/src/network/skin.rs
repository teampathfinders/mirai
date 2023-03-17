
use util::{
    bail, Serialize, Error, Result
};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use util::bytes::{BinaryReader, BinaryWriter, MutableBuffer, SharedBuffer};

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

impl TryFrom<&str> for ArmSize {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "slim" => Self::Slim,
            "wide" => Self::Wide,
            _ => bail!(Malformed, "Invalid arm size {value}")
        })
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

impl TryFrom<&str> for PersonaPieceType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "persona_skeleton" => Self::Skeleton,
            "persona_body" => Self::Body,
            "persona_skin" => Self::Skin,
            "persona_bottom" => Self::Bottom,
            "persona_feet" => Self::Feet,
            "persona_top" => Self::Top,
            "persona_mouth" => Self::Mouth,
            "persona_hair" => Self::Hair,
            "persona_eyes" => Self::Eyes,
            "persona_facial_hair" => Self::FacialHair,
            "persona_dress" => Self::Dress,
            _ => bail!(Malformed, "Invalid persona piece type '{value}'")
        })
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
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_str(&self.piece_id);
        buffer.write_str(self.piece_type.name());
        buffer.write_str(&self.pack_id);
        buffer.write_bool(self.default);
        buffer.write_str(&self.product_id);
    }

    fn deserialize(buffer: &mut SharedBuffer) -> Result<Self> {
        let piece_id = buffer.read_str()?.to_owned();
        let piece_type = PersonaPieceType::try_from(buffer.read_str()?)?;
        let pack_id = buffer.read_str()?.to_owned();
        let default = buffer.read_bool()?;
        let product_id = buffer.read_str()?.to_owned();

        Ok(Self {
            piece_id,
            piece_type,
            pack_id,
            default,
            product_id
        })
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
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_str(self.piece_type.name());

        buffer.write_u32_le(self.colors.len() as u32);
        for color in &self.colors {
            buffer.write_str(color);
        }
    }

    fn deserialize(buffer: &mut SharedBuffer) -> Result<Self> {
        let piece_type = PersonaPieceType::try_from(buffer.read_str()?)?;

        let color_count = buffer.read_u32_le()?;
        if color_count > 4 {
            bail!(Malformed, "Persona piece tint cannot have more than 4 colours, received {color_count}");
        }

        // Not sure why Rust can't infer this type...
        let mut colors: [String; 4] = Default::default();
        for i in 0..color_count {
            colors[i as usize] = buffer.read_str()?.to_owned();
        }

        Ok(Self {
            piece_type,
            colors
        })  
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

impl TryFrom<u32> for SkinAnimationType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        Ok(match value {
            0 => Self::None,
            1 => Self::Head,
            2 => Self::Body32x32,
            3 => Self::Body128x128,
            _ => bail!(Malformed, "Invalid skin animation type {value}")
        })
    }
}

/// Expression type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum SkinExpressionType {
    Linear,
    Blinking,
}

impl TryFrom<u32> for SkinExpressionType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        Ok(match value {
            0 => Self::Linear,
            1 => Self::Blinking,
            _ => bail!(Malformed, "Invalid skin expression type {value}")
        })
    }
}

/// A skin animation.
#[derive(Debug, Deserialize)]
pub struct SkinAnimation {
    /// Width of the animation image in pixels.
    #[serde(rename = "ImageWidth")]
    pub image_width: u32,
    /// Height of the animation image in pixels.
    #[serde(rename = "ImageHeight")]
    pub image_height: u32,
    /// Image data.
    #[serde(rename = "Image", with = "base64")]
    pub image_data: MutableBuffer,
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
    pub fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_u32_le(self.image_width);
        buffer.write_u32_le(self.image_height);
        
        buffer.write_var_u32(self.image_data.len() as u32);
        buffer.append(&self.image_data);

        buffer.write_u32_le(self.animation_type as u32);
        buffer.write_f32_le(self.frame_count);
        buffer.write_u32_le(self.expression_type as u32);
    }

    pub fn deserialize(buffer: &mut SharedBuffer) -> Result<Self> {
        let image_width = buffer.read_u32_le()?;
        let image_height = buffer.read_u32_le()?;
        let image_size = buffer.read_var_u32()?;
        let image_data = MutableBuffer::from(buffer.take_n(image_size as usize)?.to_vec());

        let animation_type = SkinAnimationType::try_from(buffer.read_u32_le()?)?;
        let frame_count = buffer.read_f32_le()?;
        let expression_type = SkinExpressionType::try_from(buffer.read_u32_le()?)?;

        Ok(Self {
            image_width, image_height, image_data, animation_type, frame_count, expression_type
        })
    }
}

/// A classic or persona skin.
#[derive(Debug, Deserialize)]
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
    pub image_data: MutableBuffer,
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
    pub cape_image_data: MutableBuffer,
    /// JSON containing information like bones.
    #[serde(rename = "SkinGeometryData", with = "base64_string")]
    pub geometry: String,
    #[serde(rename = "SkinAnimationData", with = "base64_string")]
    pub animation_data: String,
    /// Engine version for geometry data.
    #[serde(rename = "SkinGeometryDataEngineVersion", with = "base64_string")]
    pub geometry_engine_version: String,
    /// Whether this skin was purchased from the marketplace.
    #[serde(rename = "PremiumSkin")]
    pub is_premium: bool,
    /// Whether this skin is a persona skin.
    #[serde(rename = "PersonaSkin")]
    pub is_persona: bool,
    /// Whether the skin is classic but has a persona cape equipped.
    #[serde(rename = "CapeOnClassicSkin")]
    pub cape_on_classic_skin: bool,
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
    pub is_trusted: bool,
    #[serde(skip)]
    pub full_id: String,
    #[serde(skip)]
    pub is_primary_user: bool
}

/// Serde deserializer for raw base64.
mod base64 {
    use base64::Engine;
    use serde::{Deserializer, Deserialize};
    use util::bytes::{MutableBuffer};

    const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<MutableBuffer, D::Error> {
        let base64 = String::deserialize(d)?;

        let bytes = ENGINE.decode(base64).map_err(serde::de::Error::custom)?;
        Ok(MutableBuffer::from(bytes))
    }
}

/// Serde deserializer that decodes the base64 and converts it into a string.
mod base64_string {
    use base64::Engine;
    use serde::{Deserializer, Deserialize};

    const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        let base64 = String::deserialize(d)?;
        let bytes = ENGINE.decode(base64).map_err(serde::de::Error::custom)?;

        String::from_utf8(bytes).map_err(serde::de::Error::custom)
    }
}

impl Skin {
    /// Validates skin dimensions.
    pub fn validate(&self) -> Result<()> {
        if self.image_width * self.image_height * 4
            != self.image_data.len() as u32
        {
            bail!(
                Malformed,
                "Invalid skin dimensions, image data does not match"
            )
        }

        if self.cape_image_width * self.cape_image_height * 4
            != self.cape_image_data.len() as u32
        {
            bail!(
                Malformed,
                "Invalid cape dimensions, image data does not match"
            )
        }

        for animation in &self.animations {
            if animation.image_width * animation.image_height * 4
                != animation.image_data.len() as u32
            {
                bail!(
                    Malformed,
                    "Invalid animation dimensions, image data does not match"
                )
            }
        }

        Ok(())
    }

    pub fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_str(&self.skin_id);
        buffer.write_str(&self.playfab_id);
        buffer.write_str(&self.resource_patch);

        buffer.write_u32_le(self.image_width);
        buffer.write_u32_le(self.image_height);
        buffer.write_var_u32(self.image_data.len() as u32);
        buffer.append(self.image_data.as_ref());

        buffer.write_u32_le(self.animations.len() as u32);
        for animation in &self.animations {
            animation.serialize(buffer);
        }

        buffer.write_u32_le(self.cape_image_width);
        buffer.write_u32_le(self.cape_image_height);
        buffer.write_var_u32(self.cape_image_data.len() as u32);
        buffer.append(self.cape_image_data.as_ref());

        buffer.write_str(&self.geometry);
        buffer.write_str(&self.geometry_engine_version);
        buffer.write_str(&self.animation_data);

        buffer.write_str(&self.cape_id);
        buffer.write_str(&self.full_id);
        buffer.write_str(self.arm_size.name());
        buffer.write_str(&self.color);

        buffer.write_u32_le(self.persona_pieces.len() as u32);
        for piece in &self.persona_pieces {
            piece.serialize(buffer);
        }

        buffer.write_u32_le(self.persona_piece_tints.len() as u32);
        for tint in &self.persona_piece_tints {
            tint.serialize(buffer);
        }

        buffer.write_bool(self.is_premium);
        buffer.write_bool(self.is_persona);
        buffer.write_bool(self.cape_on_classic_skin);
        buffer.write_bool(self.is_primary_user);
    }

    pub fn deserialize(buffer: &mut SharedBuffer) -> Result<Self> {
        let skin_id = buffer.read_str()?.to_owned();
        let playfab_id = buffer.read_str()?.to_owned();
        let resource_patch = buffer.read_str()?.to_owned();
        
        let image_width = buffer.read_u32_le()?;
        let image_height = buffer.read_u32_le()?;
        let image_size = buffer.read_var_u32()?;
        let image_data = MutableBuffer::from(buffer.take_n(image_size as usize)?.to_vec());

        let animation_count = buffer.read_u32_le()?;
        let mut animations = Vec::with_capacity(animation_count as usize);
        for _ in 0..animation_count {
            animations.push(SkinAnimation::deserialize(buffer)?);
        }

        let cape_image_width = buffer.read_u32_le()?;
        let cape_image_height = buffer.read_u32_le()?;
        let cape_image_size = buffer.read_var_u32()?;
        let cape_image_data = MutableBuffer::from(buffer.take_n(cape_image_size as usize)?.to_vec());

        let geometry = buffer.read_str()?.to_owned();
        let geometry_engine_version = buffer.read_str()?.to_owned();
        let animation_data = buffer.read_str()?.to_owned();
        let cape_id = buffer.read_str()?.to_owned();
        let full_id = buffer.read_str()?.to_owned();
        let arm_size = ArmSize::try_from(buffer.read_str()?)?;
        let color = buffer.read_str()?.to_owned();

        let persona_piece_count = buffer.read_u32_le()?;
        let mut persona_pieces = Vec::with_capacity(persona_piece_count as usize);
        for _ in 0..persona_piece_count {
            persona_pieces.push(PersonaPiece::deserialize(buffer)?);            
        }

        let persona_tint_count = buffer.read_u32_le()?;
        let mut persona_piece_tints = Vec::with_capacity(persona_tint_count as usize);
        for _ in 0..persona_piece_count {
            persona_piece_tints.push(PersonaPieceTint::deserialize(buffer)?);
        }

        let is_premium = buffer.read_bool()?;
        let is_persona = buffer.read_bool()?;
        let cape_on_classic_skin = buffer.read_bool()?;
        let is_primary_user = buffer.read_bool()?;

        Ok(Self {
            skin_id, 
            playfab_id, 
            resource_patch, 
            image_width, 
            image_height, 
            image_data,
            animations,
            cape_image_width,
            cape_image_height,
            cape_image_data,
            geometry,
            geometry_engine_version,
            animation_data,
            cape_id,
            full_id,
            arm_size,
            color,
            persona_pieces,
            persona_piece_tints,
            is_premium,
            is_persona,
            cape_on_classic_skin,
            is_primary_user,
            is_trusted: false
        })
    }
}