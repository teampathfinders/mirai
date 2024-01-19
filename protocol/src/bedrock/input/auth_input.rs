use macros::variant_count;
use util::{Deserialize, SharedBuffer, BinaryRead, Vector};
use crate::bedrock::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[variant_count]
pub enum PlayMode {
    Normal,
    Teaser,
    Screen,
    Viewer,
    VirtualReality,
    Placement,
    LivingRoom,
    ExitLevel,
    ExitLevelLivingRoom
}

impl TryFrom<u32> for PlayMode {
    type Error = anyhow::Error;
    
    fn try_from(v: u32) -> anyhow::Result<Self> {
        if v <= Self::variant_count() as u32 {
            // SAFETY: This is safe because the enum has a `u32` repr and the discriminant is in range.
            Ok(unsafe { std::mem::transmute::<u32, Self>(v) })
        } else {
            anyhow::bail!("Play mode out of range ({v} > {})", Self::variant_count());
        }
    }
}

pub struct PlayerAuthInput {
    pub pitch: f32,
    pub yaw: f32,
    pub head_yaw: f32,
//    
//    pub position: Vector<f32, 3>,
//    pub moved: Vector<f32, 2>,
//    pub analogue_moved: Vector<f32, 2>,
//    
//    pub input_data: u64,
//    pub input_mode: u32,
//    pub play_mode: u32,
//    pub interaction_model: i32,
//    pub gaze_direction: Vector<f32, 3>,
//    
//    pub tick: u64,
//    pub delta: Vector<f32, 3>,
//    
//    pub item_transaction: TransactionData,
//    pub item_stack: StackRequest,
//    pub block_actions: Vec<PlayerAction>
}

impl ConnectedPacket for PlayerAuthInput {
    const ID: u32 = 0x90;
}

impl<'a> Deserialize<'a> for PlayerAuthInput {
    fn deserialize(buffer: SharedBuffer<'a>) -> anyhow::Result<Self> {
//        let pitch = buffer.read_f32_le()?;
//        let yaw = buffer.read_f32_le()?;
//        let position = buffer.read_vecf()?;
//        let moved = buffer.read_vecf()?;
//        let head_yaw = buffer.read_f32_le()?;
//        let input_data = buffer.read_var_u64()?;
//        let input_mode = buffer.read_var_u32()?;
//        let play_mode = buffer.read_var_u32()?;
//        let interaction_model = buffer.read_var_i32()?;
//        
//        if play_mode == 
        
        todo!()
    }
}