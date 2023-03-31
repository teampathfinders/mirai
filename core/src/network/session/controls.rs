use util::{Deserialize, Result};
use util::bytes::MutableBuffer;

use crate::network::{
    {
        Interact, MovePlayer,
    },
    Session,
};

impl Session {
    pub fn process_interaction(&self, pk: MutableBuffer) -> Result<()> {
        let _request = Interact::deserialize(pk.snapshot())?;

        Ok(())
    }

    pub fn process_move_player(&self, pk: MutableBuffer) -> Result<()> {
        let request = MovePlayer::deserialize(pk.snapshot())?;
        self.broadcast_others(request)?;

        Ok(())
    }
}
