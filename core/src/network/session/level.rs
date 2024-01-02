use anyhow::anyhow;
use proto::bedrock::{GameRule, ParsedCommand};
use util::{error, pyassert, Result, TryExpect};
use crate::level::LevelManager;

pub const DEFAULT_EFFECT_DURATION: i32 = 30;

impl LevelManager {
    pub fn on_gamerule_command(&self, caller: u64, command: ParsedCommand) -> anyhow::Result<String> {
        debug_assert_eq!(command.name, "gamerule");

        // Parsing should already verify that these parameters are provided.
        debug_assert!(command.parameters.contains_key("rule"));

        let rule_name = command.parameters.get("rule")
            // Rule parameter should exist, but this is here just to be sure.
            .unwrap()
            .as_string()
            .try_expect("Expected `rule` of type String")?;

        // Command has value parameter, store the game rule value.
        if let Some(value) = command.parameters.get("value") {
            let new_value = GameRule::from_parsed(rule_name, value)?;
            let old_value = self.set_game_rule(new_value)?;

            if let Some(old_value) = old_value {
                Ok(format!("Set game rule '{rule_name}' to {new_value} (was {old_value})."))
            } else {
                Ok(format!("Set game rule '{rule_name}' to {new_value} (was not set)."))
            }
        } else {
            // Command has no value parameter, load the game rule value.
            if let Some(value) = self.get_game_rule(rule_name) {
                Ok(format!("Game rule '{rule_name}' is set to {value}"))
            } else {
                Ok(format!("Game rule '{rule_name}' is not set"))
            }
        }
    }

    pub fn on_effect_command(&self, caller: u64, command: ParsedCommand) -> anyhow::Result<String> {
        debug_assert_eq!(command.name, "effect");

        // Parsing should already verify that these parameters are provided.
        debug_assert!(command.parameters.contains_key("effect"));
        debug_assert!(command.parameters.contains_key("target"));

        let effect_name = command.parameters.get("effect")
            .unwrap()
            .as_string()
            .try_expect("Expected `effect` of type String")?;

        if effect_name == "clear" {
            // TODO: Specify names of entities.
            Ok("Took all effects from entities".to_owned())
        } else {
            // If there's no duration, apply a default 30 seconds
            let duration = if let Some(duration) = command.parameters.get("duration") {
                duration.as_int().try_expect("Expected `duration` of type Int")?
            } else {
                DEFAULT_EFFECT_DURATION
            };

            let amplifier = if let Some(amplifier) = command.parameters.get("amplifier") {
                amplifier.as_int().try_expect("Expected `amplifier` of type Int")?
            } else {
                1
            };

            let hide_particles = if let Some(hide_particles) = command.parameters.get("hideParticles") {
                let h = hide_particles.as_string().try_expect("Expected `hideParticles` of type String")?;
                if h == "true" {
                    true
                } else {
                    false
                }
            } else {
                false
            };

            Ok(format!("Applied {} * {} for {} seconds", effect_name, amplifier, duration))
        }
    }
}