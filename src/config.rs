use std::sync::Arc;

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::lang::Ruleset;

/// In minutes
const DEFAULT_TEXT_DETECT_COOLDOWN: i64 = 5;

pub struct Config {
    text_detect_cooldown: Duration,
    discord_token: String,
    starboard_reaction_count: u64,
    starboard_emote_name: String,
    starboard_channel_id: u64,
    responses: Vec<MessageResponse>,
    config_path: String,
}

impl Config {
    pub fn get_cooldown(&self) -> &Duration {
        &self.text_detect_cooldown
    }

    pub fn get_starboard_reaction_count(&self) -> &u64 {
        &self.starboard_reaction_count
    }

    pub fn get_starboard_emote(&self) -> &String {
        &self.starboard_emote_name
    }

    pub fn get_starboard_channel(&self) -> &u64 {
        &self.starboard_channel_id
    }

    pub fn get_responses(&self) -> &Vec<MessageResponse> {
        &self.responses
    }

    pub fn get_config_path(&self) -> &str {
        &self.config_path
    }

    /// Fetches the config from the config.toml file in the root directory.
    pub fn create_from_file(config_path: &str) -> Config {
        let file = std::fs::read_to_string(config_path)
            .expect(format!("Could not read {}", config_path).as_str());

        let ConfigBuilder {
            text_detect_cooldown,
            discord_token,
            starboard_reaction_count,
            starboard_emote_name,
            starboard_channel_id,
            responses,
        } = toml::from_str(&file).expect("Could not deserialize config.toml");

        let text_detect_cooldown = Duration::minutes(text_detect_cooldown);

        Config {
            text_detect_cooldown,
            discord_token,
            starboard_reaction_count,
            starboard_emote_name,
            starboard_channel_id,
            responses,
            config_path: config_path.to_owned(),
        }
    }

    /// Reloads the config file and updates the configuration.
    pub fn reload(&mut self) {
        *self = Config::create_from_file(&self.config_path);
    }

    /// Updates config.toml with the new cooldown, and updates the cooldown as well
    pub fn update_cooldown(&mut self, cooldown: Duration) {
        self.text_detect_cooldown = cooldown;

        self.save();
    }

    /// Adds a response to the config.toml file and the config.
    pub fn add_response(&mut self, response: MessageResponse) {
        self.responses.push(response);
        self.save();
    }

    /// Removes a response from the config.toml file and the config.
    pub fn remove_response(&mut self, name: String) {
        self.responses.retain(|response| *response.name != name);
        self.save();
    }

    pub fn get_response(&self, name: &str) -> &MessageResponse {
        self.responses
            .iter()
            .find(|response| *response.name == name)
            .expect("Could not find response with name")
    }

    pub fn get_token(&self) -> &str {
        &self.discord_token
    }

    pub fn save(&self) {
        let config_builder = ConfigBuilder {
            text_detect_cooldown: self.text_detect_cooldown.num_minutes(),
            discord_token: self.discord_token.clone(),
            starboard_reaction_count: self.starboard_reaction_count,
            starboard_emote_name: self.starboard_emote_name.clone(),
            starboard_channel_id: self.starboard_channel_id,
            responses: self.responses.clone(),
        };

        let toml = toml::to_string(&config_builder).expect("Could not serialize config");

        std::fs::write(&self.config_path, toml).expect("Could not write to config.toml");
    }
}

fn get_default_text_detect_cooldown() -> i64 {
    DEFAULT_TEXT_DETECT_COOLDOWN
}

fn get_default_discord_token() -> String {
    std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN")
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
struct ConfigBuilder {
    #[serde(default = "get_default_text_detect_cooldown")]
    text_detect_cooldown: i64,
    #[serde(default = "get_default_discord_token")]
    discord_token: String,
    starboard_reaction_count: u64,
    starboard_emote_name: String,
    starboard_channel_id: u64,
    responses: Vec<MessageResponse>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum MessageResponseKind {
    Text { content: String },
    RandomText { content: Vec<String> },
    Image { path: String },
    TextAndImage { content: String, path: String },
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct MessageResponse {
    pub name: Arc<String>,
    pub ruleset: Ruleset,
    #[serde(flatten)]
    // This makes it so it pretends the attributes of the enum are attributes of the struct
    pub kind: MessageResponseKind,
}

#[cfg(test)]
mod test {
    use crate::fast_ruleset;

    use super::*;

    #[test]
    fn should_deserialize_properly() {
        let test_input = r#"
discord_token = "test_token_not_real"
starboard_reaction_count = 3
starboard_emote_name = "star"
starboard_channel_id = 123456789109876
[[responses]]
name = "1984"
ruleset = '''
r 1234
!r 4312
'''
content = "literally 1984""#;

        let config: ConfigBuilder = toml::from_str(test_input).unwrap();

        assert_eq!(
            config.responses.first().unwrap().ruleset,
            fast_ruleset!("r 1234", "!r 4312")
        );

        assert_eq!(
            config.responses.first(),
            Some(&MessageResponse {
                name: Arc::new("1984".to_string()),
                ruleset: fast_ruleset!("r 1234", "!r 4312"),
                kind: MessageResponseKind::Text {
                    content: "literally 1984".to_string(),
                },
            })
        );
    }
}
