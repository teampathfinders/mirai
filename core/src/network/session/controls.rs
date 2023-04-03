use util::{Deserialize};
use util::bytes::MutableBuffer;

use crate::network::{ContainerOpen, ContainerType, InteractAction};
use crate::network::{
    {
        Interact, MovePlayer,
    },
    Session,
};

impl Session {
    pub fn process_interaction(&self, pk: MutableBuffer) -> anyhow::Result<()> {
        let request = Interact::deserialize(pk.snapshot())?;
        dbg!(&request);

        if request.action == InteractAction::OpenInventory {
            self.send(ContainerOpen {
                container_type: ContainerType::Inventory,
                ..Default::default()
            })?;
        }

        Ok(())
    }

    pub fn process_move_player(&self, pk: MutableBuffer) -> anyhow::Result<()> {
        let request = MovePlayer::deserialize(pk.snapshot())?;
        dbg!(&request);

        self.broadcast_others(request)?;

        Ok(())
    }
}
