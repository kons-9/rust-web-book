use derive_new::new;
use garde::Validate;
use kernel::model::{
    id::UserId,
    role::Role,
    user::{
        User,
        event::{CreateUser, UpdateUserPassword, UpdateUserRole},
    },
};
use serde::{Deserialize, Serialize};
use strum::VariantNames;

#[derive(Serialize, Deserialize, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum RoleName {
    Admin,
    User,
}

impl From<Role> for RoleName {
    fn from(value: Role) -> Self {
        match value {
            Role::Admin => RoleName::Admin,
            Role::User => RoleName::User,
        }
    }
}
impl From<RoleName> for Role {
    fn from(value: RoleName) -> Self {
        match value {
            RoleName::Admin => Role::Admin,
            RoleName::User => Role::User,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersResponse {
    pub items: Vec<UserResponse>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub role: RoleName,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        let User {
            id,
            name,
            email,
            role,
        } = value;
        UserResponse {
            id,
            name,
            email,
            role: role.into(),
        }
    }
}

#[derive(Validate, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserPasswordRequest {
    #[garde(length(min = 1))]
    current_password: String,
    #[garde(length(min = 1))]
    new_password: String,
}

#[derive(new)]
pub struct UpdateUserPasswordRequestWithUserId {
    pub user_id: UserId,
    pub request: UpdateUserPasswordRequest,
}

impl From<UpdateUserPasswordRequestWithUserId> for UpdateUserPassword {
    fn from(value: UpdateUserPasswordRequestWithUserId) -> Self {
        let UpdateUserPasswordRequestWithUserId { user_id, request } = value;
        let UpdateUserPasswordRequest {
            current_password,
            new_password,
        } = request;
        UpdateUserPassword {
            user_id,
            current_password,
            new_password,
        }
    }
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreaterUserRequest {
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 1))]
    pub password: String,
}

impl From<CreaterUserRequest> for CreateUser {
    fn from(value: CreaterUserRequest) -> Self {
        let CreaterUserRequest {
            name,
            email,
            password,
        } = value;
        CreateUser {
            name,
            email,
            password,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRoleRequest {
    role: RoleName,
}

#[derive(new)]
pub struct UpdateUserRoleRequestWithUserId {
    pub user_id: UserId,
    pub request: UpdateUserRoleRequest,
}

impl From<UpdateUserRoleRequestWithUserId> for UpdateUserRole {
    fn from(value: UpdateUserRoleRequestWithUserId) -> Self {
        let UpdateUserRoleRequestWithUserId { user_id, request } = value;
        let UpdateUserRoleRequest { role } = request;
        UpdateUserRole {
            user_id,
            role: role.into(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookOwner {
    pub id: UserId,
    pub name: String,
}

impl From<kernel::model::user::BookOwner> for BookOwner {
    fn from(value: kernel::model::user::BookOwner) -> Self {
        let kernel::model::user::BookOwner { id, name } = value;
        BookOwner { id, name }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutUser {
    pub id: UserId,
    pub name: String,
}

impl From<kernel::model::user::CheckoutUser> for CheckoutUser {
    fn from(value: kernel::model::user::CheckoutUser) -> Self {
        let kernel::model::user::CheckoutUser { id, name } = value;
        CheckoutUser { id, name }
    }
}
