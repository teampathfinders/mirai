use std::{collections::HashMap, sync::Arc};

use futures::{future, StreamExt};
use level::{BiomeEncoding, BiomeStorage, Biomes, SubChunk, SubStorage};
use proto::{
    bedrock::{
        Animate, CommandOutput, CommandOutputMessage, CommandOutputType, CommandRequest, DisconnectReason, FormResponseData, HeightmapType,
        HudElement, HudVisibility, InventoryTransaction, ItemInstance, LevelChunk, MobEquipment, NetworkChunkPublisherUpdate, PlayerAuthInput,
        RequestAbility, SetHud, SetInventoryOptions, SettingsCommand, SubChunkEntry, SubChunkRequestMode, SubChunkResponse, SubChunkResult, TextData,
        TextMessage, TickSync, TransactionAction, TransactionSourceType, TransactionType, UpdateSkin, WindowId,
    },
    types::Dimension,
};

use util::{BinaryRead, BinaryWrite, CowSlice, Deserialize, RVec};

use crate::level::io::r#box::BoxRegion;
use crate::level::io::stream::IndexedSubChunk;

use super::BedrockClient;

impl BedrockClient {
    /// Handles a mob equipment packet.
    pub fn handle_mob_equipment(&self, packet: RVec) -> anyhow::Result<()> {
        let equipment = MobEquipment::deserialize(packet.as_ref())?;

        // Verify that runtime ID matches player's runtime ID.
        // Clients only send this packet to modify themselves.
        if equipment.runtime_id != self.runtime_id()? {
            // Illegal packet modifications
            self.kick_with_reason("Illegal packets", DisconnectReason::BadPacket)?;
        }

        self.broadcast_others(equipment)
    }

    pub fn handle_inventory_options(&self, packet: RVec) -> anyhow::Result<()> {
        let options = SetInventoryOptions::deserialize(packet.as_ref())?;
        tracing::debug!("{options:?}");

        Ok(())
    }

    pub fn handle_inventory_transaction(&self, packet: RVec) -> anyhow::Result<()> {
        let transaction = InventoryTransaction::deserialize(packet.as_ref())?;
        tracing::debug!("{transaction:?}");
        // let action = &transaction.actions[0];
        // let item = &action.new_item;

        // let transaction = InventoryTransaction {
        //     legacy_request_id: 0,
        //     legacy_transactions: vec![],
        //     transaction_type: TransactionType::Normal,
        //     actions: vec![
        //         TransactionAction {
        //             slot: 0,
        //             source_type: TransactionSourceType::Container {
        //                 inventory_id: WindowId::Ui
        //             },
        //             new_item: ItemInstance::air(),
        //             old_item: item.clone()
        //         },
        //         TransactionAction {
        //             slot: 2,
        //             source_type: TransactionSourceType::Container {
        //                 inventory_id: WindowId::Hotbar
        //             },
        //             old_item: ItemInstance::air(),
        //             new_item: item.clone()
        //         }
        //     ]
        // };
        // self.send(transaction)?;

        // for action in transaction.actions {
        //     let instance = self.instance();

        //     let new = instance.item_network_ids.get_name(action.new_item.network_id);
        //     // let old = instance.item_network_ids.get_name(action.old_item.network_id);

        //     let mut buf = Vec::with_capacity(5);
        //     buf.write_var_i32(action.new_item.network_id)?;

        //     let mut var = buf.as_slice();
        //     let var = var.read_var_u32()?;
        //     println!("{var}")

        //     // println!("Switch from {old:?} to {new:?}");
        // }

        Ok(())
    }

    /// Handles a [`SettingsCommand`] packet used to adjust a world setting.
    pub fn handle_settings_command(&self, packet: RVec) -> anyhow::Result<()> {
        let request = SettingsCommand::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");

        Ok(())
    }

    /// Handles a [`TickSync`] packet used to synchronise ticks between the client and server.
    pub fn handle_tick_sync(&self, packet: RVec) -> anyhow::Result<()> {
        let _request = TickSync::deserialize(packet.as_ref())?;
        // TODO: Implement tick synchronisation
        Ok(())
        // let response = TickSync {
        //     request_tick: request.request_tick,
        //     response_tick: self.level.
        // };
        // self.send(response)
    }

    /// Handles a [`TextMessage`] packet sent when a client wants to send a chat message.
    #[tracing::instrument(
        skip_all,
        name = "BedrockUser::handle_text_message"
        fields(
            username = %self.name().unwrap_or("<unknown>"),
            msg
        )
    )]
    pub fn handle_text_message(self: &Arc<Self>, packet: RVec) -> anyhow::Result<()> {
        let request = TextMessage::deserialize(packet.as_ref())?;
        if let TextData::Chat { source, message } = request.data {
            tracing::Span::current().record("msg", message);

            let name = self.name()?;
            // Check that the source is equal to the player name to prevent spoofing.
            if name != source {
                tracing::warn!("Client and text message name do not match. Kicking them for forbidden modifications");
                return self.kick_with_reason("Illegal packet modifications detected", DisconnectReason::BadPacket);
            }

            // We must also return the packet to the client that sent it
            // otherwise their message won't be displayed in their own chat.
            self.broadcast(request)
        } else {
            // Only the server is allowed to create text packets that are not of the chat type.
            tracing::warn!("Client sent an illegal message type. Kicking them for forbidden modifications");
            self.kick_with_reason("Illegal packet received", DisconnectReason::BadPacket)
        }
    }

    /// Handles a [`PlayerAuthInput`] packet. These are sent every tick and are used
    /// for server authoritative player movement.
    pub fn handle_auth_input(&self, packet: RVec) -> anyhow::Result<()> {
        let input = PlayerAuthInput::deserialize(packet.as_ref())?;
        if input.input_data.0 != 0 {
            // tracing::debug!("{:?}", input.input_data);
        }
        
        Ok(())
    }

    /// Handles an [`UpdateSkin`] packet.
    pub fn handle_skin_update(&self, packet: RVec) -> anyhow::Result<()> {
        let request = UpdateSkin::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");
        self.broadcast(request)
    }

    /// Handles an [`AbilityRequest`] packet.
    pub fn handle_ability_request(&self, packet: RVec) -> anyhow::Result<()> {
        let request = RequestAbility::deserialize(packet.as_ref())?;
        tracing::debug!("{request:?}");

        Ok(())
    }

    /// Handles an [`Animation`] packet.
    pub fn handle_animation(&self, packet: RVec) -> anyhow::Result<()> {
        let request = Animate::deserialize(packet.as_ref())?;

        let transaction = InventoryTransaction {
            legacy_request_id: 0,
            legacy_transactions: vec![],
            transaction_type: TransactionType::Normal,
            actions: vec![
                TransactionAction {
                    slot: 0,
                    source_type: TransactionSourceType::Container { inventory_id: WindowId::Creative },
                    new_item: ItemInstance::air(),
                    old_item: ItemInstance {
                        block_runtime_id: 13256,
                        network_id: 5,
                        blocking_tick: 0,
                        can_destroy: vec![],
                        can_place_on: vec![],
                        count: 12,
                        metadata: 0,
                        nbt: HashMap::new(),
                        stack_id: None,
                    },
                },
                TransactionAction {
                    slot: 0,
                    source_type: TransactionSourceType::Container { inventory_id: WindowId::Ui },
                    old_item: ItemInstance::air(),
                    new_item: ItemInstance {
                        block_runtime_id: 13256,
                        network_id: 5,
                        blocking_tick: 0,
                        can_destroy: vec![],
                        can_place_on: vec![],
                        count: 12,
                        metadata: 0,
                        nbt: HashMap::new(),
                        stack_id: None,
                    },
                },
            ],
        };
        self.send(transaction)?;

        let transaction = InventoryTransaction {
            legacy_request_id: 0,
            legacy_transactions: vec![],
            transaction_type: TransactionType::Normal,
            actions: vec![
                TransactionAction {
                    slot: 0,
                    source_type: TransactionSourceType::Container { inventory_id: WindowId::Ui },
                    new_item: ItemInstance::air(),
                    old_item: ItemInstance {
                        block_runtime_id: 13256,
                        network_id: 5,
                        blocking_tick: 0,
                        can_destroy: vec![],
                        can_place_on: vec![],
                        count: 12,
                        metadata: 0,
                        nbt: HashMap::new(),
                        stack_id: None,
                    },
                },
                TransactionAction {
                    slot: 0,
                    source_type: TransactionSourceType::Container { inventory_id: WindowId::Inventory },
                    old_item: ItemInstance::air(),
                    new_item: ItemInstance {
                        block_runtime_id: 13256,
                        network_id: 5,
                        blocking_tick: 0,
                        can_destroy: vec![],
                        can_place_on: vec![],
                        count: 12,
                        metadata: 0,
                        nbt: HashMap::new(),
                        stack_id: None,
                    },
                },
            ],
        };
        self.send(transaction)?;

        tracing::debug!("{request:?}");

        Ok(())
    }

    /// Handles a [`FormResponseData`] packet. This packet is forwarded to the forms [`Subscriber`](crate::forms::response::Subscriber)
    /// which will properly handle the response.
    ///
    /// # Errors
    ///
    /// May return an error if the packet fails to deserialize or handling a form response fails.
    pub fn handle_form_response(&self, packet: RVec) -> anyhow::Result<()> {
        let response = FormResponseData::deserialize(packet.as_ref())?;
        self.forms.handle_response(response)
    }

    /// Handles a [`CommandRequest`] packet.
    ///
    /// # Errors
    ///
    /// May return an error if the packet fails to deserialize or executing the command fails.
    #[tracing::instrument(
        skip_all,
        name = "handle_command_request",
        fields(
            command,
            username = self.name().unwrap_or("<unknown>")
        )
    )]
    pub fn handle_command_request(self: Arc<Self>, packet: RVec) {
        // let this = self.clone();
        // tokio::spawn(async move {
        //     let stream = this.viewer.service.region(BoxRegion::from_bounds(
        //         [0, -4, 0], [0, 15, 0], Dimension::Overworld
        //     ));

        //     stream.take(1).for_each(|sub| {
        //         dbg!(sub);
        //         future::ready(())
        //     }).await;
        // });

        // Request the chunk the player is in
        let stream = self
            .viewer
            .service
            .region(BoxRegion::from_bounds((0, -4, 0), (0, 15, 0), Dimension::Overworld));

        self.send(NetworkChunkPublisherUpdate { position: (0, 0, 0).into(), radius: 12 }).unwrap();

        // let this = self.clone();
        // tokio::spawn(async move {
        //     let fut = stream.take(1).for_each(|res| {
        //         tracing::debug!("{res:?}");

        //         let chunk = res.data;

        //         let mut ser = chunk.serialize_network(&this.instance().block_states).unwrap();

        //         // No biomes
        //         let indices = Box::new([0u16; 4096]);
        //         let storage = BiomeStorage {
        //             indices, palette: vec![0]
        //         };

        //         let biome = Biomes {
        //             fragments: vec![BiomeEncoding::Paletted(storage)],
        //             heightmap: Box::new([[0; 16]; 16])
        //         };

        //         biome.serialize(&mut ser).unwrap();

        //         // Border block count
        //         ser.write_u8(0).unwrap();

        //         // let de = SubChunk::deserialize_network(ser).unwrap();
        //         // tracing::debug!("{de:?}");

        //         this.send(LevelChunk {
        //             request_mode: SubChunkRequestMode::Limitless,
        //             coordinates: (0, 0).into(),
        //             dimension: Dimension::Overworld,
        //             sub_chunk_count: 24,
        //             highest_sub_chunk: 16,
        //             blob_hashes: None,
        //             raw_payload: ser
        //         }).unwrap();

        //         // this.send(SubChunkResponse {
        //         //     cache_enabled: false,
        //         //     dimension: Dimension::Overworld,
        //         //     entries: vec![SubChunkEntry {
        //         //         blob_hash: 0,
        //         //         payload: ser,
        //         //         offset: (0, 0, 0).into(),
        //         //         result: SubChunkResult::Success,
        //         //         heightmap: Box::new([[0; 16]; 16]),
        //         //         heightmap_type: HeightmapType::None
        //         //     }],
        //         //     position: (0, 0, 0).into()
        //         // }).unwrap();

        //         future::ready(())
        //     });

        //     fut.await;
        // });

        // self.viewer.update_radius(12);

        // Command execution could take several ticks, await the result in a separate task
        // to avoid blocking the request handler.
        tokio::spawn(async move {
            let request = match CommandRequest::deserialize(packet.as_ref()) {
                Ok(req) => req,
                Err(err) => {
                    tracing::error!("Failed to deserialize `CommandRequest`: {err:#}");
                    return;
                }
            };
            tracing::Span::current().record("command", request.command);

            let receiver = match self.commands.execute(Arc::clone(&self), request.command.to_owned()).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("{e:#}");
                    return;
                }
            };

            receiver.await.map_or_else(
                |_| tracing::error!("Command service shut down while awaiting execution"),
                |result| {
                    let is_success = result.is_ok();
                    let data = match result {
                        Ok(r) => r,
                        Err(r) => r,
                    };

                    let messages = vec![CommandOutputMessage {
                        is_success,
                        message: data.message,
                        parameters: CowSlice::Owned(data.parameters),
                    }];

                    let output = CommandOutput {
                        success_count: if is_success { 1 } else { 0 },
                        request_id: request.request_id,
                        origin: request.origin,
                        output_type: CommandOutputType::AllOutput,
                        output: CowSlice::Owned(messages),
                    };

                    if let Err(err) = self.send(output) {
                        tracing::error!("Failed to send command output to client: {err:#}");
                    }
                },
            );
        });
    }
}
