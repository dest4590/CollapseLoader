use std::{path::PathBuf, sync::Mutex};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::api::core::data::DATA;

use super::common::JsonStorage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub last_used: Option<String>,
    pub is_active: bool,
}

impl Account {
    pub fn new(username: String, tags: Vec<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            tags,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
            is_active: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountManager {
    pub accounts: Vec<Account>,
    pub active_account_id: Option<String>,
    pub accounts_path: PathBuf,
}

impl AccountManager {
    pub fn add_account(&mut self, username: String, tags: Vec<String>) -> String {
        let account = Account::new(username, tags);
        let id = account.id.clone();
        self.accounts.push(account);
        id
    }

    pub fn remove_account(&mut self, id: &str) -> bool {
        if let Some(pos) = self.accounts.iter().position(|a| a.id == id) {
            self.accounts.remove(pos);
            if self.active_account_id.as_ref() == Some(&id.to_string()) {
                self.active_account_id = None;
            }
            true
        } else {
            false
        }
    }

    pub fn set_active_account(&mut self, id: &str) -> bool {
        if self.accounts.iter().any(|a| a.id == id) {
            for acc in &mut self.accounts {
                acc.is_active = false;
            }

            if let Some(account) = self.accounts.iter_mut().find(|a| a.id == id) {
                account.is_active = true;
                account.last_used = Some(chrono::Utc::now().to_rfc3339());
            }

            self.active_account_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    pub fn get_active_account(&self) -> Option<&Account> {
        self.accounts.iter().find(|a| a.is_active)
    }

    pub fn update_account(
        &mut self,
        id: &str,
        username: Option<String>,
        tags: Option<Vec<String>>,
    ) -> bool {
        if let Some(account) = self.accounts.iter_mut().find(|a| a.id == id) {
            if let Some(new_username) = username {
                account.username = new_username;
            }
            if let Some(new_tags) = tags {
                account.tags = new_tags;
            }
            true
        } else {
            false
        }
    }
}

impl JsonStorage for AccountManager {
    fn file_path(&self) -> &PathBuf {
        &self.accounts_path
    }

    fn resource_name() -> &'static str {
        "accounts"
    }

    fn create_default() -> Self {
        Self::default()
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            active_account_id: None,
            accounts_path: DATA.get_local("accounts.json"),
        }
    }
}

lazy_static! {
    pub static ref ACCOUNT_MANAGER: Mutex<AccountManager> = Mutex::new(
        AccountManager::load_from_disk(DATA.get_local("accounts.json"))
    );
}
