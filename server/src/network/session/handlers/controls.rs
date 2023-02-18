use bytes::BytesMut;
use common::{Deserialize, VResult, Vector3i};

use crate::network::{
    packets::{
        CameraShake, CameraShakeAction, CameraShakeType, Interact,
        InteractAction, MovePlayer, PlaySound,
    },
    session::Session,
};

impl Session {
    pub fn handle_interaction(&self, packet: BytesMut) -> VResult<()> {
        let request = Interact::deserialize(packet)?;

        Ok(())
    }

    pub fn handle_move_player(&self, packet: BytesMut) -> VResult<()> {
        let request = MovePlayer::deserialize(packet)?;

        Ok(())
    }
}
