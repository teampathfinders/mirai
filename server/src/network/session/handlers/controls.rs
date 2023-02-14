use bytes::BytesMut;
use common::{Decodable, VResult, Vector3i};

use crate::network::{
    packets::{
        CameraShake, CameraShakeAction, CameraShakeType, Interact,
        InteractAction, MovePlayer, PlaySound,
    },
    session::Session,
};

impl Session {
    pub fn handle_interaction(&self, packet: BytesMut) -> VResult<()> {
        let request = Interact::decode(packet)?;
        tracing::info!("{request:?}");

        if request.action == InteractAction::OpenInventory {
            let reply = CameraShake {
                action: CameraShakeAction::Add,
                duration: 0.5,
                intensity: 0.25,
                shake_type: CameraShakeType::Rotational,
            };
            self.send_packet(reply)?;

            let reply2 = PlaySound {
                name: "mob.pig.say",
                pitch: 1.0,
                volume: 1.0,
                position: Vector3i::from([0, 0, 0]),
            };
            self.send_packet(reply2)?;
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
