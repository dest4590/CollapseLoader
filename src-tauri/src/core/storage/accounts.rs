use super::common::JsonStorage;
use crate::core::auth::microsoft::MinecraftAccountData;
use crate::core::storage::data::DATA;
use crate::log_error;
use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

const MICROSOFT_ACCESS_TOKEN_SERVICE: &str = "CollapseLoader Microsoft access token";
const MICROSOFT_REFRESH_TOKEN_SERVICE: &str = "CollapseLoader Microsoft refresh token";

fn default_has_refresh_token() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    #[default]
    Offline,
    Microsoft,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub last_used: Option<String>,
    pub is_active: bool,
    #[serde(default)]
    pub account_type: AccountType,
    #[serde(default)]
    pub uuid: Option<String>,
    // These fields are deserialized only to migrate accounts created by older versions.
    // They are never written back to disk, even if secure-store migration fails.
    #[serde(default, skip_serializing)]
    access_token: Option<String>,
    #[serde(default, skip_serializing)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
    #[serde(default = "default_has_refresh_token")]
    has_refresh_token: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicAccount {
    pub id: String,
    pub username: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub last_used: Option<String>,
    pub is_active: bool,
    pub account_type: AccountType,
    pub uuid: Option<String>,
}

impl From<&Account> for PublicAccount {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            username: account.username.clone(),
            tags: account.tags.clone(),
            created_at: account.created_at.clone(),
            last_used: account.last_used.clone(),
            is_active: account.is_active,
            account_type: account.account_type.clone(),
            uuid: account.uuid.clone(),
        }
    }
}

fn run_credential_operation<T>(
    operation: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String>
where
    T: Send + 'static,
{
    // Linux Secret Service uses an internal Tokio runtime in the synchronous keyring
    // adapter. Running it on a Tauri/Tokio worker would panic due to nested runtimes.
    std::thread::Builder::new()
        .name("credential-store".to_string())
        .spawn(operation)
        .map_err(|error| format!("Failed to start credential-store worker: {error}"))?
        .join()
        .map_err(|_| "Credential-store worker panicked.".to_string())?
}

fn credential_entry(service: &str, account_id: &str) -> Result<Entry, String> {
    Entry::new(service, account_id)
        .map_err(|error| format!("System credential store is unavailable: {error}"))
}

fn set_credential_blocking(service: &str, account_id: &str, secret: &str) -> Result<(), String> {
    credential_entry(service, account_id)?
        .set_password(secret)
        .map_err(|error| format!("Failed to save credentials securely: {error}"))
}

fn get_credential_blocking(service: &str, account_id: &str) -> Result<String, String> {
    credential_entry(service, account_id)?
        .get_password()
        .map_err(|error| match error {
            KeyringError::NoEntry => {
                "Microsoft credentials are missing. Please sign in again.".to_string()
            }
            _ => format!("Failed to read credentials from the system store: {error}"),
        })
}

fn delete_credential_blocking(service: &str, account_id: &str) -> Result<(), String> {
    match credential_entry(service, account_id)?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(error) => Err(format!(
            "Failed to remove credentials from the system store: {error}"
        )),
    }
}

fn get_credential(service: &str, account_id: &str) -> Result<String, String> {
    let service = service.to_string();
    let account_id = account_id.to_string();
    run_credential_operation(move || get_credential_blocking(&service, &account_id))
}

fn store_microsoft_credentials(
    account_id: &str,
    access_token: &str,
    refresh_token: Option<&str>,
) -> Result<(), String> {
    let account_id = account_id.to_string();
    let access_token = access_token.to_string();
    let refresh_token = refresh_token.map(str::to_string);
    run_credential_operation(move || {
        set_credential_blocking(MICROSOFT_ACCESS_TOKEN_SERVICE, &account_id, &access_token)?;
        if let Some(refresh_token) = refresh_token.as_deref() {
            set_credential_blocking(MICROSOFT_REFRESH_TOKEN_SERVICE, &account_id, refresh_token)?;
        } else {
            delete_credential_blocking(MICROSOFT_REFRESH_TOKEN_SERVICE, &account_id)?;
        }

        let stored_access = get_credential_blocking(MICROSOFT_ACCESS_TOKEN_SERVICE, &account_id)?;
        if stored_access != access_token {
            return Err(
                "The system credential store did not verify the saved access token.".to_string(),
            );
        }
        if let Some(refresh_token) = refresh_token.as_deref() {
            let stored_refresh =
                get_credential_blocking(MICROSOFT_REFRESH_TOKEN_SERVICE, &account_id)?;
            if stored_refresh != refresh_token {
                return Err(
                    "The system credential store did not verify the saved refresh token."
                        .to_string(),
                );
            }
        }
        Ok(())
    })
}

fn delete_microsoft_credentials(account_id: &str) -> Result<(), String> {
    let account_id = account_id.to_string();
    run_credential_operation(move || {
        delete_credential_blocking(MICROSOFT_ACCESS_TOKEN_SERVICE, &account_id)?;
        delete_credential_blocking(MICROSOFT_REFRESH_TOKEN_SERVICE, &account_id)
    })
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
            account_type: AccountType::Offline,
            uuid: None,
            access_token: None,
            refresh_token: None,
            expires_at: None,
            has_refresh_token: false,
        }
    }

    fn from_microsoft(id: String, data: &MinecraftAccountData) -> Self {
        Self {
            id,
            username: data.username.clone(),
            tags: vec!["Microsoft".to_string()],
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
            is_active: false,
            account_type: AccountType::Microsoft,
            uuid: Some(data.uuid.clone()),
            access_token: None,
            refresh_token: None,
            expires_at: Some(data.expires_at.clone()),
            has_refresh_token: data.refresh_token.is_some(),
        }
    }

    pub fn minecraft_data(&self) -> Result<Option<MinecraftAccountData>, String> {
        if self.account_type != AccountType::Microsoft {
            return Ok(None);
        }

        let access_token = match &self.access_token {
            Some(token) => token.clone(),
            None => get_credential(MICROSOFT_ACCESS_TOKEN_SERVICE, &self.id)?,
        };
        let refresh_token = match &self.refresh_token {
            Some(token) => Some(token.clone()),
            None if self.has_refresh_token => {
                Some(get_credential(MICROSOFT_REFRESH_TOKEN_SERVICE, &self.id)?)
            }
            None => None,
        };

        Ok(Some(MinecraftAccountData {
            username: self.username.clone(),
            uuid: self
                .uuid
                .clone()
                .ok_or_else(|| "Microsoft account UUID is missing.".to_string())?,
            access_token,
            refresh_token,
            expires_at: self
                .expires_at
                .clone()
                .ok_or_else(|| "Microsoft account expiration is missing.".to_string())?,
        }))
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
        let mut loaded = <Self as JsonStorage>::load_from_disk_with(path.clone(), |manager| {
            manager.accounts_path = path;
        });
        loaded.migrate_plaintext_microsoft_credentials();
        loaded.remove_plaintext_sidecars_if_safe();
        loaded
    }

    fn remove_plaintext_sidecars_if_safe(&self) {
        let still_contains_plaintext = self
            .accounts
            .iter()
            .any(|account| account.access_token.is_some() || account.refresh_token.is_some());
        if still_contains_plaintext {
            return;
        }

        let _ = std::fs::remove_file(self.accounts_path.with_extension("bak"));
        let _ = std::fs::remove_file(self.accounts_path.with_extension("json.tmp"));
    }

    fn migrate_plaintext_microsoft_credentials(&mut self) {
        let mut found_plaintext = false;
        for account in &mut self.accounts {
            if account.account_type != AccountType::Microsoft
                || (account.access_token.is_none() && account.refresh_token.is_none())
            {
                continue;
            }
            found_plaintext = true;

            match (&account.access_token, &account.refresh_token) {
                (Some(access_token), Some(refresh_token)) => {
                    if let Err(error) =
                        store_microsoft_credentials(&account.id, access_token, Some(refresh_token))
                    {
                        log_error!(
                            "Could not migrate Microsoft account {} to secure storage; sign-in will be required again: {}",
                            account.id,
                            error
                        );
                    }
                }
                _ => log_error!(
                    "Microsoft account {} had incomplete legacy credentials; sign-in will be required again",
                    account.id
                ),
            }

            // Fail closed: plaintext tokens are removed even when the system store
            // is unavailable. The user can sign in again, but tokens never remain on disk.
            account.access_token = None;
            account.refresh_token = None;
        }
        if found_plaintext {
            self.save_to_disk();
        }
    }

    pub fn add_account(&mut self, username: String, tags: Vec<String>) -> String {
        let account = Account::new(username, tags);
        let id = account.id.clone();
        self.accounts.push(account);
        self.save_to_disk();
        id
    }

    pub fn add_microsoft_account(&mut self, data: MinecraftAccountData) -> Result<String, String> {
        if let Some(index) = self.accounts.iter().position(|account| {
            account.account_type == AccountType::Microsoft
                && account.uuid.as_deref() == Some(data.uuid.as_str())
        }) {
            let id = self.accounts[index].id.clone();
            let has_refresh_token = data.refresh_token.is_some();
            store_microsoft_credentials(&id, &data.access_token, data.refresh_token.as_deref())?;
            let existing = &mut self.accounts[index];
            existing.username = data.username;
            existing.expires_at = Some(data.expires_at);
            existing.has_refresh_token = has_refresh_token;
            existing.access_token = None;
            existing.refresh_token = None;
            self.save_to_disk();
            return Ok(id);
        }

        let id = uuid::Uuid::new_v4().to_string();
        store_microsoft_credentials(&id, &data.access_token, data.refresh_token.as_deref())?;
        self.accounts
            .push(Account::from_microsoft(id.clone(), &data));
        self.save_to_disk();
        Ok(id)
    }

    pub fn update_microsoft_account(
        &mut self,
        id: &str,
        data: MinecraftAccountData,
    ) -> Result<(), String> {
        let Some(index) = self.accounts.iter().position(|account| account.id == id) else {
            return Err("Microsoft account not found.".to_string());
        };
        if self.accounts[index].account_type != AccountType::Microsoft {
            return Err("Cannot store Microsoft credentials for an offline account.".to_string());
        }

        let has_refresh_token = data.refresh_token.is_some();
        store_microsoft_credentials(id, &data.access_token, data.refresh_token.as_deref())?;
        let account = &mut self.accounts[index];
        account.username = data.username;
        account.uuid = Some(data.uuid);
        account.expires_at = Some(data.expires_at);
        account.has_refresh_token = has_refresh_token;
        account.access_token = None;
        account.refresh_token = None;
        self.save_to_disk();
        Ok(())
    }

    pub fn remove_account(&mut self, id: &str) -> Result<bool, String> {
        let Some(position) = self.accounts.iter().position(|account| account.id == id) else {
            return Ok(false);
        };
        if self.accounts[position].account_type == AccountType::Microsoft {
            delete_microsoft_credentials(id)?;
        }

        self.accounts.remove(position);
        if self.active_account_id.as_deref() == Some(id) {
            self.active_account_id = None;
        }
        self.save_to_disk();
        Ok(true)
    }

    pub fn set_active_account(&mut self, id: &str) -> bool {
        if !self.accounts.iter().any(|account| account.id == id) {
            return false;
        }
        for account in &mut self.accounts {
            account.is_active = false;
        }
        if let Some(account) = self.accounts.iter_mut().find(|account| account.id == id) {
            account.is_active = true;
            account.last_used = Some(chrono::Utc::now().to_rfc3339());
        }
        self.active_account_id = Some(id.to_string());
        self.save_to_disk();
        true
    }

    pub fn get_active_account(&self) -> Option<&Account> {
        self.accounts.iter().find(|account| account.is_active)
    }

    pub fn update_account(
        &mut self,
        id: &str,
        username: Option<String>,
        tags: Option<Vec<String>>,
    ) -> bool {
        if let Some(account) = self.accounts.iter_mut().find(|account| account.id == id) {
            if account.account_type == AccountType::Offline {
                if let Some(new_username) = username {
                    account.username = new_username;
                }
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

    pub fn reorder_accounts(&mut self, ordered_ids: Vec<String>) -> bool {
        let mut reordered: Vec<Account> = Vec::with_capacity(ordered_ids.len());
        for id in &ordered_ids {
            if let Some(account) = self.accounts.iter().find(|account| &account.id == id) {
                reordered.push(account.clone());
            }
        }
        for account in &self.accounts {
            if !ordered_ids.contains(&account.id) {
                reordered.push(account.clone());
            }
        }
        self.accounts = reordered;
        self.save_to_disk();
        true
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

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_microsoft_account() -> Account {
        let mut account = Account::new("Player".to_string(), vec!["Microsoft".to_string()]);
        account.account_type = AccountType::Microsoft;
        account.uuid = Some("minecraft-uuid".to_string());
        account.access_token = Some("minecraft-token".to_string());
        account.refresh_token = Some("microsoft-refresh-token".to_string());
        account.expires_at = Some("2099-01-01T00:00:00Z".to_string());
        account
    }

    #[test]
    fn legacy_credentials_are_read_but_never_reserialized() {
        let legacy_json = serde_json::json!({
            "id": "legacy-id",
            "username": "Player",
            "tags": ["Microsoft"],
            "created_at": "2026-01-01T00:00:00Z",
            "last_used": null,
            "is_active": true,
            "account_type": "microsoft",
            "uuid": "minecraft-uuid",
            "access_token": "minecraft-token",
            "refresh_token": "microsoft-refresh-token",
            "expires_at": "2099-01-01T00:00:00Z"
        });
        let account: Account = serde_json::from_value(legacy_json).unwrap();
        assert_eq!(account.access_token.as_deref(), Some("minecraft-token"));
        assert_eq!(
            account.refresh_token.as_deref(),
            Some("microsoft-refresh-token")
        );

        let saved = serde_json::to_value(account).unwrap();
        assert!(saved.get("access_token").is_none());
        assert!(saved.get("refresh_token").is_none());
    }

    #[test]
    fn account_metadata_never_contains_credentials() {
        let json = serde_json::to_value(legacy_microsoft_account()).unwrap();
        assert!(json.get("access_token").is_none());
        assert!(json.get("refresh_token").is_none());
    }

    #[test]
    fn public_account_does_not_expose_credentials() {
        let account = legacy_microsoft_account();
        let json = serde_json::to_value(PublicAccount::from(&account)).unwrap();
        assert!(json.get("access_token").is_none());
        assert!(json.get("refresh_token").is_none());
        assert!(json.get("expires_at").is_none());
    }

    #[tokio::test]
    async fn credential_worker_runs_outside_tokio_runtime() {
        let is_outside_runtime =
            run_credential_operation(|| Ok(tokio::runtime::Handle::try_current().is_err()))
                .unwrap();
        assert!(is_outside_runtime);
    }
}
