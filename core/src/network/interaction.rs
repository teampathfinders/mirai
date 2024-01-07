use std::sync::atomic::Ordering;

use proto::bedrock::{ABILITY_FLYING, ABILITY_MAYFLY, ABILITY_MUTED, AbilityData, AbilityLayer, AbilityType, ContainerClose, ContainerOpen, ContainerType, GameMode, Interact, InteractAction, INVENTORY_WINDOW_ID, MovementMode, MovePlayer, PlayerAction, PlayerActionType, UpdateAbilities};
use util::{MutableBuffer, Deserialize};

use super::BedrockUser;

impl BedrockUser {
    pub fn process_interaction(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = Interact::deserialize(packet.snapshot())?;
        if request.action == InteractAction::OpenInventory {
            if !self.player().is_inventory_open.fetch_or(true, Ordering::Relaxed) {
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
            self.player().is_inventory_open.store(false, Ordering::Relaxed);

            // The server also needs to send a container close packet back.
            self.send(ContainerClose {
                window_id: INVENTORY_WINDOW_ID,
                ..Default::default()
            })?;
        }

        Ok(())
    }

    pub async fn process_move_player(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let mut request = MovePlayer::deserialize(packet.snapshot())?;

        self.replicator.move_player(self.xuid(), &request).await?;

        request.mode = MovementMode::Normal;

        self.broadcast(request)
    }

    pub fn process_player_action(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = PlayerAction::deserialize(packet.snapshot())?;
        // dbg!(&request);

        if request.action == PlayerActionType::StartFlying {
            // Only allow flying if the player is in the correct gamemode.
            let gamemode = self.player().gamemode();
            if gamemode == GameMode::Creative || gamemode == GameMode::Spectator {
                self.send(UpdateAbilities {
                    data: AbilityData {
                        command_permission_level: self.player().command_permission_level(),
                        permission_level: self.player().permission_level(),
                        unique_id: self.player().runtime_id(),
                        layers: vec![
                            AbilityLayer {
                                fly_speed: 0.05,
                                walk_speed: 0.1,
                                values: ABILITY_MAYFLY | ABILITY_FLYING | ABILITY_MUTED,
                                abilities: ABILITY_MAYFLY | ABILITY_FLYING | ABILITY_MUTED,
                                ability_type: AbilityType::Base
                            }
                        ]
                    },
                })?;
            }
        }

        Ok(())
    }
}
