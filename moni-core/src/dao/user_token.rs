use anyhow::Result;
use jwt_simple::algorithms::MACLike;
use jwt_simple::prelude::{Claims, Duration, HS256Key};
use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::FromRow)]
struct UserToken {
    token: String,
    user_id: i64,
    salt: String,
}

#[derive(Serialize, Deserialize)]
struct UserJwtToken {
    token: String,
    user_id: i64,
}

async fn get_by_token(token: &str) -> Result<UserToken> {
    let pool = crate::DB.get().unwrap();
    let result = sqlx::query_as::<_, UserToken>(
        "SELECT `token`,`user_id`,`salt` FROM user_token WHERE token = ? AND status = ?",
    )
    .bind(token)
    .bind(1)
    .fetch_one(pool)
    .await?;
    Ok(result)
}

/// create_jwt_token check token and generate jwt token
pub async fn create_jwt_token(token: &str) -> Result<String> {
    let result = get_by_token(token).await?;
    // generate jwt token with salt
    let key = HS256Key::from_bytes(result.salt.as_bytes());
    let jwt_token_data = UserJwtToken {
        token: result.token,
        user_id: result.user_id,
    };
    // create a jwt token for on three months
    let claims = Claims::with_custom_claims(jwt_token_data, Duration::from_days(90));
    let token = key.authenticate(claims)?;
    Ok(token)
}

/// verify_jwt_token check jwt token
pub async fn verify_jwt_token(token: &str, jwt_token: &str) -> Result<bool> {
    let result = get_by_token(token).await?;
    let key = HS256Key::from_bytes(result.salt.as_bytes());
    let claims = key.verify_token::<UserJwtToken>(jwt_token, None)?;
    Ok(claims.custom.token == result.token && claims.custom.user_id == result.user_id)
}
