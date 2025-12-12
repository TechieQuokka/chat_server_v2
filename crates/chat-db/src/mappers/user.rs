//! User entity <-> model mapper

use chat_core::entities::User;
use chat_core::value_objects::Snowflake;

use crate::models::UserModel;

/// Convert UserModel to User entity
impl From<UserModel> for User {
    fn from(model: UserModel) -> Self {
        User {
            id: Snowflake::new(model.id),
            username: model.username,
            discriminator: model.discriminator,
            email: model.email,
            avatar: model.avatar,
            bot: model.bot,
            system: model.system,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Convert User entity reference to values for database insertion/update
pub struct UserInsert<'a> {
    pub id: i64,
    pub username: &'a str,
    pub discriminator: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub avatar: Option<&'a str>,
    pub bot: bool,
    pub system: bool,
}

impl<'a> UserInsert<'a> {
    pub fn new(user: &'a User, password_hash: &'a str) -> Self {
        Self {
            id: user.id.into_inner(),
            username: &user.username,
            discriminator: &user.discriminator,
            email: &user.email,
            password_hash,
            avatar: user.avatar.as_deref(),
            bot: user.bot,
            system: user.system,
        }
    }
}

/// Convert User entity reference to values for database update
pub struct UserUpdate<'a> {
    pub id: i64,
    pub username: &'a str,
    pub avatar: Option<&'a str>,
}

impl<'a> UserUpdate<'a> {
    pub fn new(user: &'a User) -> Self {
        Self {
            id: user.id.into_inner(),
            username: &user.username,
            avatar: user.avatar.as_deref(),
        }
    }
}
