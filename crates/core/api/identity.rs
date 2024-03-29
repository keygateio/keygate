use std::sync::Arc;

use crate::{
    database::{models, DatabasePool},
    KeygateInternal,
};

use keygate_utils::{
    random::secure_random_id,
    validate::{is_valid_email, is_valid_password, is_valid_username, validate_field},
};

use super::{APIError, Filter, SortBy, SortOrder, UserIdentifier};

#[derive(Debug, Clone)]
pub struct Identity {
    keygate: Arc<KeygateInternal>,
}

#[derive(Debug, Clone)]
pub struct CreateIdentity<'a> {
    pub username: Option<&'a str>,
    pub primary_email: Option<&'a str>,
    pub password_hash: Option<&'a str>,
}

const USERNAME_REQUIRED: bool = true;
const EMAIL_REQUIRED: bool = true;
const PASSWORD_REQUIRED: bool = true;

impl Identity {
    pub(crate) fn new(keygate: Arc<KeygateInternal>) -> Self {
        Self { keygate }
    }

    fn db(&self) -> &DatabasePool {
        &self.keygate.db
    }

    pub async fn exists(
        &self,
        // everything with an @ is considered an email
        username_or_email: &str,
    ) -> Result<bool, APIError> {
        let user = match username_or_email.contains('@') {
            true => sqlx::query!(
                "SELECT id FROM Identity WHERE primary_email = $1",
                username_or_email
            )
            .fetch_optional(self.db())
            .await?
            .map(|x| x.id),
            false => sqlx::query!(
                "SELECT id FROM Identity WHERE username = $1",
                username_or_email
            )
            .fetch_optional(self.db())
            .await?
            .map(|x| x.id),
        };

        Ok(user.is_some())
    }

    pub async fn get(&self, user: UserIdentifier) -> Result<Option<models::Identity>, APIError> {
        let (field, value) = match user {
            UserIdentifier::Email(email) => ("primary_email", email),
            UserIdentifier::Username(username) => ("username", username),
            UserIdentifier::Id(id) => ("id", id),
        };

        let identity = sqlx::query_as!(
            models::Identity,
            "SELECT * FROM Identity WHERE $1 = $2",
            field,
            value
        )
        .fetch_optional(self.db())
        .await?;

        Ok(identity)
    }

    pub async fn create<'a>(
        &self,
        identity: CreateIdentity<'a>,
    ) -> Result<models::Identity, APIError> {
        let user_id = secure_random_id();
        let email_token = secure_random_id();
        let now = time::OffsetDateTime::now_utc();
        let email_expires_at = now + time::Duration::minutes(15);

        validate_field(
            &identity.username,
            USERNAME_REQUIRED,
            is_valid_username,
            APIError::invalid_argument("Invalid username"),
        )?;

        validate_field(
            &identity.primary_email,
            EMAIL_REQUIRED,
            is_valid_email,
            APIError::invalid_argument("Invalid email"),
        )?;
        validate_field(
            &identity.password_hash,
            PASSWORD_REQUIRED,
            is_valid_password,
            APIError::invalid_argument("Invalid password"),
        )?;

        let mut tx = self.db().begin().await?;

        let mut identity = sqlx::query_as!(
            models::Identity,
            r#"
                INSERT INTO Identity (id, username, password_hash, created_at, updated_at, last_active)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    RETURNING *;
            "#,
            user_id,
            identity.username,
            identity.password_hash,
            now,
            now,
            now,
        )
        .fetch_one(&mut *tx)
        .await?;

        if let Some(email) = identity.primary_email.clone() {
            sqlx::query!(
                "INSERT INTO Email (email, identity_id, verified, verification_code, verification_code_expires_at, created_at, updated_at)
                    VALUES ($1, $2, false, $3, $4, $5, $6)",
                email,
                user_id,
                email_token,
                email_expires_at,
                now,
                now
            )
            .execute(&mut *tx)
            .await?;

            identity = sqlx::query_as!(
                models::Identity,
                "UPDATE Identity SET primary_email = $1 WHERE id = $2 RETURNING *",
                identity.primary_email,
                user_id
            )
            .fetch_one(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(identity)
    }

    pub async fn update(
        &self,
        update: impl FnOnce(models::Identity) -> models::Identity,
        id: &str,
    ) -> Result<models::Identity, APIError> {
        let now = time::OffsetDateTime::now_utc();
        let mut tx = self.db().begin().await?;

        let identity =
            sqlx::query_as!(models::Identity, "SELECT * FROM Identity WHERE id = $1", id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(APIError::not_found("User not found"))?;

        let identity = update(identity.clone());

        if USERNAME_REQUIRED && identity.username.is_none() {
            return Err(APIError::invalid_argument("Invalid username"));
        }

        if identity
            .username
            .clone()
            .is_some_and(|u| !is_valid_username(&u))
        {
            return Err(APIError::invalid_argument("Invalid username"));
        }

        sqlx::query!(
            "UPDATE Identity SET updated_at = $1, username = $2 WHERE id = $4",
            now,
            identity.username,
            id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(identity)
    }

    pub async fn delete_permanent(&self, id: &str) -> Result<(), APIError> {
        sqlx::query!("DELETE FROM Identity WHERE id = $1", id)
            .execute(self.db())
            .await?;

        Ok(())
    }

    pub async fn list(
        &self,
        filter: Filter,
        sort_by: SortBy,
        sort_order: SortOrder,
        offset: u32,
        count: u32,
    ) -> Result<Vec<models::Identity>, APIError> {
        if count > 100 {
            return Err(APIError::invalid_argument(
                "Count cannot be greater than 100",
            ));
        }

        let order_field = match sort_by {
            SortBy::Email => "primary_email",
            SortBy::Username => "username",
            SortBy::CreatedAt => "created_at",
            SortBy::LastActive => "last_active",
        };

        let (filter_field, filter_value) = match filter {
            Filter { ref field, value } if field == "username" => ("username", value),
            Filter { ref field, value } if field == "primary_email" => ("primary_email", value),
            _ => ("id", "".to_string()),
        };

        let identities =
            match sort_order {
                SortOrder::Asc => sqlx::query_as!(
                    models::Identity,
                    r#"SELECT * FROM Identity WHERE $1 LIKE $2 ORDER BY $3 ASC LIMIT $4 OFFSET $5"#,
                    filter_field,
                    filter_value,
                    order_field,
                    count,
                    offset
                )
                .fetch_all(self.db())
                .await?,
                SortOrder::Desc => {
                    sqlx::query_as!(
                    models::Identity,
                    "SELECT * FROM Identity WHERE $1 LIKE $2 ORDER BY $3 DESC LIMIT $4 OFFSET $5",
                    filter_field,
                    filter_value,
                    order_field,
                    count,
                    offset
                )
                    .fetch_all(self.db())
                    .await?
                }
            };

        Ok(identities)
    }
}
