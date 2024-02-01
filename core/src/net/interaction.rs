use std::sync::atomic::Ordering;

use proto::bedrock::{ABILITY_FLYING, AbilityData, AbilityLayer, AbilityType, ContainerClose, ContainerOpen, ContainerType, GameMode, Interact, InteractAction, INVENTORY_WINDOW_ID, MovePlayer, PlayerAction, PlayerActionType, UpdateAbilities, ABILITY_FLAG_END};
use util::Deserialize;

use super::BedrockUser;

impl BedrockUser {
    pub fn handle_interaction(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = Interact::deserialize(packet.as_ref())?;
      
        if request.action == InteractAction::OpenInventory && !self.player().is_inventory_open.fetch_or(true, Ordering::Relaxed) {
            self.send(ContainerOpen {
                window_id: INVENTORY_WINDOW_ID,
                container_type: ContainerType::Inventory,
                ..Default::default()
            })?;
        }

        Ok(())
    }

    pub fn handle_container_close(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = ContainerClose::deserialize(packet.as_ref())?;
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

    pub async fn handle_move_player(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let _request = MovePlayer::deserialize(packet.as_ref())?;

        Ok(())
        // self.replicator.move_player(self.xuid(), &request).await?;

        // request.mode = MovementMode::Normal;

        // self.broadcast(request)
    }

    pub fn handle_player_action(&self, packet: Vec<u8>) -> anyhow::Result<()> {
        let request = PlayerAction::deserialize(packet.as_ref())?;
        
        match request.action {
            PlayerActionType::StartFlying => self.action_start_flying(request),
            PlayerActionType::StopFlying => self.action_stop_flying(request),
            _ => Ok(())
        }
    }

    // Actions
    // ======================================================================================

    #[inline]
    fn action_start_flying(&self, _action: PlayerAction) -> anyhow::Result<()> {
        // Only allow flying if the player is in the correct gamemode.
        let gamemode = self.player().gamemode();
        if gamemode == GameMode::Creative || gamemode == GameMode::Spectator {
            self.send(UpdateAbilities(
                AbilityData {
                    command_permission_level: self.player().command_permission_level(),
                    permission_level: self.player().permission_level(),
                    unique_id: self.player().runtime_id(),
                    layers: vec![
                        AbilityLayer {
                            fly_speed: 0.05,
                            walk_speed: 0.1,
                            values: ABILITY_FLYING,
                            abilities: ABILITY_FLAG_END - 1,
                            ability_type: AbilityType::Base
                        }
                    ]
                },
            ))?;
        }

        Ok(())
    }

    #[inline]
    fn action_stop_flying(&self, _action: PlayerAction) -> anyhow::Result<()> {
        self.send(UpdateAbilities(
            AbilityData {
                command_permission_level: self.player().command_permission_level,
                permission_level: self.player().permission_level(),
                unique_id: self.player().runtime_id(),
                layers: vec![
                    AbilityLayer {
                        fly_speed: 0.05,
                        walk_speed: 0.1,
                        values: 0,
                        abilities: ABILITY_FLAG_END - 1,
                        ability_type: AbilityType::Base
                    }
                ]
            }
        ))?;

        Ok(())
    }

    // ======================================================================================
}
