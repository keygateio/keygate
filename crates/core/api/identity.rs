use crate::{models, KeygateConfigInternal, KeygateError, KeygateStorage};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("unknown error {0}")]
    Unknown(String),
}

pub struct Identity {
    config: KeygateConfigInternal,
    storage: KeygateStorage,
}

impl Identity {
    pub async fn new(config: KeygateConfigInternal, storage: KeygateStorage) -> Self {
        Self { config, storage }
    }
}

impl Identity {
    pub async fn update_in_place<F>(
        &self,
        identity_id: &str,
        closure: F,
    ) -> Result<Option<models::Identity>, KeygateError>
    where
        F: FnOnce(Option<models::Identity>) -> Option<models::Identity>,
    {
        let identity = self.get_id(identity_id).await?;
        match closure(identity) {
            Some(new_identity) => {
                if new_identity.id != identity_id {
                    return Err(IdentityError::Unknown("identity id mismatch".to_string()).into());
                }

                self.update(&new_identity).await?;
                Ok(Some(new_identity))
            }
            None => Ok(None),
        }
    }

    pub async fn get_id(&self, identity_id: &str) -> Result<Option<models::Identity>, KeygateError> {
        Ok(self.storage.get_identity_by_id(identity_id).await?)
    }

    pub async fn get_email(&self, email: &str) -> Result<Option<models::Identity>, KeygateError> {
        Ok(self.storage.get_identity_by_email(email).await?)
    }

    pub async fn get_username(&self, username: &str) -> Result<Option<models::Identity>, KeygateError> {
        Ok(self.storage.get_identity_by_username(username).await?)
    }

    pub async fn delete(&self, _identity_id: &str) -> Result<(), KeygateError> {
        todo!()
    }

    pub async fn update(&self, identity: &models::Identity) -> Result<(), KeygateError> {
        Ok(self.storage.update_identity(identity).await?)
    }

    pub async fn identities(&self) -> Result<(), KeygateError> {
        todo!()
    }
}
