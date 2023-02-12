use bytes::BytesMut;
use common::VResult;

use crate::network::{
    packets::{Interact, InteractAction, MovePlayer},
    session::Session,
    Decodable,
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
