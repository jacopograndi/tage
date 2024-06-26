use bincode::config::Configuration;
use renet::transport::NETCODE_USER_DATA_BYTES;
use serde::{Deserialize, Serialize};

use crate::get_data_dir;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, bincode::Encode, bincode::Decode)]
pub struct Member {
    pub name: String,
    pub color: [u8; 3],
    pub symbol: String,
    pub civilization: String,
}

impl Default for Member {
    fn default() -> Self {
        Member {
            name: "Nameless One".to_string(),
            color: [200, 200, 200],
            symbol: "x".to_string(),
            civilization: "Britons".to_string(),
        }
    }
}

impl Member {
    pub fn from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Self {
        bincode::decode_from_slice::<Self, Configuration>(user_data, Configuration::default())
            .unwrap()
            .0
    }

    pub fn to_user_data(&self) -> [u8; NETCODE_USER_DATA_BYTES] {
        let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
        let mut bin =
            bincode::encode_to_vec::<&Member, Configuration>(self, Configuration::default())
                .unwrap();
        bin.truncate(NETCODE_USER_DATA_BYTES);
        while bin.len() < NETCODE_USER_DATA_BYTES {
            bin.push(0)
        }
        user_data.copy_from_slice(bin.as_slice());
        user_data
    }

    pub fn disk_path() -> Option<std::path::PathBuf> {
        let mut path = get_data_dir()?;
        path.push("profile.ron");
        tracing::trace!(target: "profile", "path: {:?}", path);
        Some(path)
    }

    pub fn from_disk() -> Option<Self> {
        let Some(path) = Self::disk_path() else {
            return None;
        };

        let Ok(raw) = std::fs::read_to_string(path) else {
            return None;
        };

        ron::from_str(raw.as_str()).ok()
    }

    pub fn to_disk(&self) {
        let Some(path) = Self::disk_path() else {
            return;
        };

        let string = ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default()).unwrap();
        std::fs::write(path, string).unwrap();
    }
}
