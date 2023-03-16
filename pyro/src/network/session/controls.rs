
use util::{Deserialize, Result, Vector3i};
use util::bytes::{MutableBuffer, SharedBuffer};

use crate::{
    {
        CameraShake, CameraShakeAction, CameraShakeType, Interact,
        InteractAction, MovePlayer, PlaySound,
    },
    Session,
};

impl Session {
    pub fn handle_interaction(&self, pk: MutableBuffer) -> Result<()> {
        let request = Interact::deserialize(pk.snapshot())?;

        Ok(())
    }

    pub fn handle_move_player(&self, pk: MutableBuffer) -> Result<()> {
        let request = MovePlayer::deserialize(pk.snapshot())?;
        self.broadcast_others(request)?;

        Ok(())
    }
}
