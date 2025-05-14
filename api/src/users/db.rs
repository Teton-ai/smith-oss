use crate::middlewares::authorization::{self, AuthorizationConfig};
use anyhow::Result;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user_id: i32,
    permissions: Vec<authorization::Permission>,
}

impl CurrentUser {
    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        self.permissions
            .iter()
            .any(|permission| permission.resource == resource && permission.action == action)
    }

    pub async fn build(
        pg_pool: &PgPool,
        authorization: &AuthorizationConfig,
        auth0_sub: &str,
    ) -> Result<Self> {
        let user_id = match sqlx::query!(
            r#"
                SELECT id
                FROM auth.users
                WHERE auth0_user_id = $1
                "#,
            auth0_sub
        )
        .fetch_optional(pg_pool)
        .await?
        {
            Some(record) => record.id,
            None => {
                // Insert the user
                sqlx::query!(
                    r#"
                        INSERT INTO auth.users (auth0_user_id)
                        VALUES ($1)
                        ON CONFLICT (auth0_user_id) DO NOTHING
                        "#,
                    auth0_sub,
                )
                .execute(pg_pool)
                .await?;

                // Now fetch the ID of the newly inserted user
                sqlx::query!(
                    r#"
                        SELECT id
                        FROM auth.users
                        WHERE auth0_user_id = $1
                        "#,
                    auth0_sub
                )
                .fetch_one(pg_pool)
                .await?
                .id
            }
        };

        struct UserRole {
            role: String,
        }

        let mut user_roles = sqlx::query_as!(
            UserRole,
            r#"
                    SELECT users_roles.role
                    FROM auth.users
                    LEFT JOIN auth.users_roles ON users_roles.user_id = users.id
                    WHERE users.auth0_user_id = $1
                "#,
            auth0_sub
        )
        .fetch_all(pg_pool)
        .await
        .expect("expected user roles");

        let user_permissions = user_roles
            .iter_mut()
            .filter_map(|user_role| authorization.roles.get(&user_role.role))
            .flat_map(|role| role.permissions.clone())
            .collect();

        let current_user = CurrentUser {
            user_id,
            permissions: user_permissions,
        };

        Ok(current_user)
    }
}
