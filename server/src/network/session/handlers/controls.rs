use bytes::BytesMut;
use common::{Decodable, VResult};

use crate::network::{
    packets::{Interact, InteractAction, MovePlayer},
    session::Session,
};

impl Session {
    pub fn handle_interaction(&self, packet: BytesMut) -> VResult<()> {
        let request = Interact::decode(packet)?;
        tracing::info!("{request:?}");

        if request.action == InteractAction::OpenInventory {
            self.kick("You are not allowed to open your inventory")?;
        }

        Ok(())
    }

    pub fn handle_move_player(&self, packet: BytesMut) -> VResult<()> {
        let request = MovePlayer::decode(packet)?;

        // tracing::info!("{request:?}");

        // self.kick("Yes")
        Ok(())
    }
}
