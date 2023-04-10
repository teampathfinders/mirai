use util::{Deserialize};
use util::bytes::MutableBuffer;

use crate::network::{ContainerOpen, ContainerType, InteractAction, ContainerClose, INVENTORY_WINDOW_ID};
use crate::network::{
    {
        Interact, MovePlayer,
    },
    Session,
};

use super::SessionLike;

impl Session {
    pub fn process_interaction(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = Interact::deserialize(packet.snapshot())?;
        if request.action == InteractAction::OpenInventory {
            let mut lock = self.player.write();
            if !lock.is_inventory_open {
                lock.is_inventory_open = true;
                drop(lock);

                self.send(ContainerOpen {
                    window_id: INVENTORY_WINDOW_ID,
                    container_type: ContainerType::Inventory,
                    ..Default::default()
                })?;
            }
        }

        Ok(())
    }

    pub fn process_container_close(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = ContainerClose::deserialize(packet.snapshot())?;
        if request.window_id == INVENTORY_WINDOW_ID {
            self.player.write().is_inventory_open = false;

            // The server also needs to send a container close packet back.
            self.send(ContainerClose {
                window_id: INVENTORY_WINDOW_ID,
                ..Default::default()
            })?;
        }

        Ok(())
    }

    pub fn process_move_player(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = MovePlayer::deserialize(packet.snapshot())?;
        // dbg!(&request);

        self.broadcast_others(request)?;

        Ok(())
    }
}
