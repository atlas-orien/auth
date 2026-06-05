use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::auth;
use repo::table::users::{UpdateUser, UsersService};
use uuid::Uuid;

use crate::dto::auth::{
    AuthUser, LoginRequest, RegisterRequest, UpdatePasswordRequest, UpdateProfileRequest,
};

pub struct AuthApi {
    users_svc: UsersService,
}

impl AuthApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            users_svc: UsersService::new(db),
        }
    }

    /// 注册新用户（校验 username/email 唯一性，哈希密码）
    pub async fn register(&self, req: RegisterRequest) -> BizResult<AuthUser> {
        // 检查 username 是否已存在
        if self
            .users_svc
            .find_by_username(&req.username)
            .await?
            .is_some()
        {
            return Err(BizError::new(
                auth::USERNAME_ALREADY_EXISTS,
                format!("User with username '{}' already exists", req.username),
            ));
        }

        // 检查 email 是否已存在
        if let Some(ref email) = req.email
            && self.users_svc.find_by_email(email).await?.is_some()
        {
            return Err(BizError::new(
                auth::EMAIL_ALREADY_EXISTS,
                format!("User with email '{}' already exists", email),
            ));
        }

        let hashed = hash_password(&req.password)?;

        let user = self
            .users_svc
            .create(repo::table::users::CreateUser {
                username: req.username,
                display_name: req.display_name,
                avatar: req.avatar,
                password: hashed,
                email: req.email,
            })
            .await?;

        Ok(into_auth_user(user))
    }

    /// 登录（identifier 可以是 username 或 email）
    pub async fn login(&self, req: LoginRequest) -> BizResult<AuthUser> {
        let user = self
            .find_user_by_identifier(&req.identifier)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    auth::USER_NOT_FOUND,
                    format!("User not found: {}", req.identifier),
                )
            })?;

        if user.disabled {
            return Err(BizError::new(
                auth::USER_DISABLED,
                "User account is disabled".into(),
            ));
        }

        verify_password(&req.password, &user.password)?;

        Ok(into_auth_user(user))
    }

    /// 通过 identifier 获取用户信息（identifier 可以是 username 或 email）
    pub async fn get_user_by_identifier(&self, identifier: &str) -> BizResult<AuthUser> {
        let user = self
            .find_user_by_identifier(identifier)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    auth::USER_NOT_FOUND,
                    format!("User not found: {}", identifier),
                )
            })?;

        Ok(into_auth_user(user))
    }

    /// 获取用户信息
    pub async fn get_user(&self, id: &str) -> BizResult<AuthUser> {
        let id = parse_user_id(id)?;
        let user = self.users_svc.find_by_id(id).await?.ok_or_else(|| {
            BizError::new(auth::USER_NOT_FOUND, format!("User not found: {}", id))
        })?;

        Ok(into_auth_user(user))
    }

    /// 通过 display_user_id 获取用户信息
    pub async fn get_user_by_display_user_id(&self, display_user_id: &str) -> BizResult<AuthUser> {
        let user = self
            .users_svc
            .find_by_display_user_id(display_user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    auth::USER_NOT_FOUND,
                    format!("User not found: {}", display_user_id),
                )
            })?;

        Ok(into_auth_user(user))
    }

    /// 修改密码（需要验证旧密码）
    pub async fn update_password(&self, id: &str, req: UpdatePasswordRequest) -> BizResult<()> {
        let id = parse_user_id(id)?;
        let user = self.users_svc.find_by_id(id).await?.ok_or_else(|| {
            BizError::new(auth::USER_NOT_FOUND, format!("User not found: {}", id))
        })?;

        verify_password(&req.old_password, &user.password)?;

        let hashed = hash_password(&req.new_password)?;
        self.users_svc
            .update(
                id,
                UpdateUser {
                    password: Some(hashed),
                    ..Default::default()
                },
            )
            .await?;

        Ok(())
    }

    /// 修改邮箱（清空 email_verified）
    pub async fn update_email(&self, id: &str, email: Option<String>) -> BizResult<()> {
        let id = parse_user_id(id)?;
        // 检查新 email 是否被其他用户占用
        if let Some(ref new_email) = email
            && let Some(existing) = self.users_svc.find_by_email(new_email).await?
            && existing.id != id
        {
            return Err(BizError::new(
                auth::EMAIL_ALREADY_EXISTS,
                format!("User with email '{}' already exists", new_email),
            ));
        }

        self.users_svc
            .update(
                id,
                UpdateUser {
                    email: Some(email),
                    email_verified: Some(false),
                    ..Default::default()
                },
            )
            .await?;

        Ok(())
    }

    /// 更新用户基础资料
    pub async fn update_profile(&self, id: &str, req: UpdateProfileRequest) -> BizResult<()> {
        let id = parse_user_id(id)?;
        self.users_svc
            .update(
                id,
                UpdateUser {
                    display_name: req.display_name,
                    avatar: req.avatar,
                    ..Default::default()
                },
            )
            .await?;

        Ok(())
    }

    /// 标记邮箱已验证
    pub async fn verify_email(&self, id: &str) -> BizResult<()> {
        let id = parse_user_id(id)?;
        self.users_svc
            .update(
                id,
                UpdateUser {
                    email_verified: Some(true),
                    ..Default::default()
                },
            )
            .await?;

        Ok(())
    }

    /// 启用/禁用用户
    pub async fn set_disabled(&self, id: &str, disabled: bool) -> BizResult<()> {
        let id = parse_user_id(id)?;
        self.users_svc
            .update(
                id,
                UpdateUser {
                    disabled: Some(disabled),
                    ..Default::default()
                },
            )
            .await?;

        Ok(())
    }

    /// 删除用户
    pub async fn delete_user(&self, id: &str) -> BizResult<()> {
        let id = parse_user_id(id)?;
        // 确认用户存在
        self.users_svc.find_by_id(id).await?.ok_or_else(|| {
            BizError::new(auth::USER_NOT_FOUND, format!("User not found: {}", id))
        })?;

        self.users_svc.delete(id).await
    }

    async fn find_user_by_identifier(
        &self,
        identifier: &str,
    ) -> BizResult<Option<repo::table::users::User>> {
        Ok(self
            .users_svc
            .find_by_username(identifier)
            .await?
            .or(self.users_svc.find_by_email(identifier).await?))
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn hash_password(password: &str) -> BizResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| BizError::new(auth::PASSWORD_HASH_FAILED, e.to_string()))?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, hash: &str) -> BizResult<()> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| BizError::new(auth::PASSWORD_HASH_PARSE_FAILED, e.to_string()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| BizError::new(auth::PASSWORD_INVALID, "Invalid password".into()))
}

fn into_auth_user(user: repo::table::users::User) -> AuthUser {
    AuthUser {
        id: user.id.to_string(),
        display_user_id: user.display_user_id,
        username: user.username,
        display_name: user.display_name,
        avatar: user.avatar,
        email: user.email,
        email_verified: user.email_verified,
        disabled: user.disabled,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

fn parse_user_id(id: &str) -> BizResult<Uuid> {
    Uuid::parse_str(id)
        .map_err(|_| BizError::new(auth::USER_NOT_FOUND, format!("User not found: {}", id)))
}
