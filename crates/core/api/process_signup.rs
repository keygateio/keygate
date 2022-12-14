use std::collections::HashMap;

use crate::{
    config,
    models::{self, BaseProcess, IdentityEmail, Process, UsernameEmailSignupProcess},
    utils::{self},
    KeygateConfigInternal, KeygateError, KeygateStorage,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignupError {
    #[error("unknown error")]
    Unknown,

    #[error("process not found")]
    ProcessNotFound,

    #[error("process expired")]
    ProcessExpired,

    #[error("process already completed")]
    ProcessAlreadyCompleted,

    #[error("invalid device id")]
    InvalidDeviceId,

    #[error("invalid email")]
    InvalidEmail,

    #[error("invalid password")]
    InvalidPassword,

    #[error("invalid username")]
    InvalidUsername,

    #[error("this user already exists")]
    UserAlreadyExists,
}

pub struct Signup {
    config: KeygateConfigInternal,
    storage: KeygateStorage,
}

impl Signup {
    pub async fn new(config: KeygateConfigInternal, storage: KeygateStorage) -> Self {
        Self { config, storage }
    }

    fn get_config(&self) -> Result<config::Configuration, SignupError> {
        Ok(self.config.read().map_err(|_| SignupError::Unknown)?.clone())
    }
}

impl Signup {
    pub async fn init_signup_process(
        &self,
        username: Option<String>,
        email: Option<String>,
        device_id: &str,
    ) -> Result<BaseProcess<UsernameEmailSignupProcess>, KeygateError> {
        let config = self.get_config()?;

        if !utils::validate::is_valid_id(device_id) {
            return Err(SignupError::Unknown.into());
        }

        if config.identity.signup_with_email {
            let Some(email) = email.clone() else {
                return Err(SignupError::InvalidEmail.into());
            };

            match self.storage.get_identity_by_email(&email).await {
                Err(_) => return Err(SignupError::Unknown.into()),
                Ok(Some(user)) => return Err(SignupError::UserAlreadyExists.into()),
                Ok(None) => {}
            }
        }

        if config.identity.signup_require_username {
            let Some(username) = username.clone() else {
                return Err(SignupError::InvalidUsername.into());
            };

            match self.storage.get_identity_by_username(&username).await {
                Err(_) => return Err(SignupError::Unknown.into()),
                Ok(Some(user)) => return Err(SignupError::UserAlreadyExists.into()),
                Ok(None) => {}
            }
        }

        let email = email.map(|email| {
            (
                email,
                IdentityEmail {
                    verified: false,
                    verified_at: None,
                },
            )
        });

        let process = models::BaseProcess {
            completed_at: None,
            process: models::UsernameEmailSignupProcess {
                device_id: device_id.to_string(),
                username,
                email,
            },
            id: utils::random::secure_random_id(),
            created_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now()
                .timestamp()
                .checked_add(config.identity.signup_process_lifetime)
                .ok_or(SignupError::Unknown)?,
        };

        self.storage
            .create_process(&models::Process::UsernameEmailSignup(process.clone()))
            .await
            .map_err(|_| SignupError::Unknown)?;

        Ok(process)
    }

    pub async fn finish_signup_process(
        &self,
        password: &str,
        process_id: &str,
        device_id: &str,
    ) -> Result<models::Identity, KeygateError> {
        let Some(signup_process) = self.storage.process_by_id(process_id).await.map_err(|_| SignupError::Unknown)? else {
            return Err(SignupError::ProcessNotFound.into());
        };

        let Process::UsernameEmailSignup(signup_process) = signup_process else {
            return Err(SignupError::ProcessNotFound.into());
        };

        if signup_process.process.device_id != device_id {
            return Err(SignupError::InvalidDeviceId.into());
        }

        if signup_process.expires_at < chrono::Utc::now().timestamp() {
            return Err(SignupError::ProcessExpired.into());
        }

        if signup_process.completed_at.is_some() {
            return Err(SignupError::ProcessAlreadyCompleted.into());
        }

        let emails = if let Some(email) = signup_process.process.email.clone() {
            HashMap::from_iter(vec![(email.0, email.1)])
        } else {
            HashMap::new()
        };

        let password_hash = utils::hash::password(password).map_err(|_| SignupError::Unknown)?;

        let new_identity = models::Identity {
            first_name: None,
            last_name: None,
            username: signup_process.process.username,
            emails,
            linked_accounts: HashMap::new(),
            password_hash: Some(password_hash),
            id: utils::random::secure_random_id(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        if self.storage.create_identity(&new_identity).await.is_err() {
            return Err(SignupError::Unknown.into());
        };

        Ok(new_identity)
    }

    pub async fn init_oidc_signup_process(&self, email: String) -> Result<(), KeygateError> {
        unimplemented!()
    }
}
