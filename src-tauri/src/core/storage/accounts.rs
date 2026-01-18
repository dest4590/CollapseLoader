use super::common::JsonStorage;
use crate::core::storage::data::DATA;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

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
    #[serde(skip)]
    pub accounts_path: PathBuf,
}

impl AccountManager {
    pub fn load_from_disk(path: PathBuf) -> Self {
        let mut loaded = <Self as JsonStorage>::load_from_disk(path.clone());
        loaded.accounts_path = path;
        loaded
    }

    pub fn add_account(&mut self, username: String, tags: Vec<String>) -> String {
        let account = Account::new(username, tags);
        let id = account.id.clone();
        self.accounts.push(account);
        self.save_to_disk();
        id
    }

    pub fn remove_account(&mut self, id: &str) -> bool {
        if let Some(pos) = self.accounts.iter().position(|a| a.id == id) {
            self.accounts.remove(pos);
            if self.active_account_id.as_deref() == Some(id) {
                self.active_account_id = None;
            }
            self.save_to_disk();
            true
        } else {
            false
        }
    }

    pub fn set_active_account(&mut self, id: &str) -> bool {
        if !self.accounts.iter().any(|a| a.id == id) {
            return false;
        }

        for acc in &mut self.accounts {
            acc.is_active = false;
        }

        if let Some(account) = self.accounts.iter_mut().find(|a| a.id == id) {
            account.is_active = true;
            account.last_used = Some(chrono::Utc::now().to_rfc3339());
        }

        self.active_account_id = Some(id.to_string());
        self.save_to_disk();
        true
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
            self.save_to_disk();
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
        Self {
            accounts: Vec::new(),
            active_account_id: None,
            accounts_path: DATA.get_local("accounts.json"),
        }
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::create_default()
    }
}

pub static ACCOUNT_MANAGER: LazyLock<Mutex<AccountManager>> = LazyLock::new(|| {
    Mutex::new(AccountManager::load_from_disk(
        DATA.get_local("accounts.json"),
    ))
});
