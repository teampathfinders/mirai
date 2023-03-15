
use util::{Deserialize, Result, Vector3i};

use crate::network::{
    packets::{
        CameraShake, CameraShakeAction, CameraShakeType, Interact,
        InteractAction, MovePlayer, PlaySound,
    },
    session::Session,
};

impl Session {
    pub fn handle_interaction(&self, pk: SharedBuffer) -> Result<()> {
        let request = Interact::deserialize(pk)?;

        Ok(())
    }

    pub fn handle_move_player(&self, packet: SharedBuffer) -> Result<()> {
        let request = MovePlayer::deserialize(packet)?;
        self.broadcast_others(request)?;

        Ok(())
    }
}
