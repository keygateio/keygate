use crate::{models::LoginFlow, KeygateConfigInternal, KeygateError, KeygateStorage};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("unknown error")]
    Unknown,
}

pub struct Login {
    config: KeygateConfigInternal,
    storage: KeygateStorage,
}

impl Login {
    pub fn new(config: KeygateConfigInternal, storage: KeygateStorage) -> Self {
        Self { config, storage }
    }
}

impl Login {
    pub fn login_init_flow(&self, device_id: &str) -> Result<LoginFlow, KeygateError> {
        todo!()
    }

    pub fn login_get_flow(
        &self,
        device_id: &str,
        login_flow_id: &str,
    ) -> Result<LoginFlow, KeygateError> {
        todo!()
    }

    pub fn login_submit_password(
        &self,
        login_flow_id: &str,
        password: &str,
    ) -> Result<LoginFlow, KeygateError> {
        todo!()
    }

    pub fn login_submit_magic_link(
        &self,
        login_flow_id: &str,
        password: &str,
    ) -> Result<LoginFlow, KeygateError> {
        todo!()
    }

    pub fn login_submit_otp(
        &self,
        login_flow_id: &str,
        device_id: &str,
        magic_link: &str,
    ) -> Result<LoginFlow, KeygateError> {
        todo!()
    }
}