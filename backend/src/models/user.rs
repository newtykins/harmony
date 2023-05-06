use async_graphql::{Context, Error, Object, SimpleObject};
use chrono::Utc;
use harmony::{get_db, SNOWFLAKE_EPOCH};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use pwhash::bcrypt;
use rustflake::Snowflake;
use serde::Serialize;

/// the user id generator
static mut SNOWFLAKE: Lazy<Snowflake> = Lazy::new(|| Snowflake::new(SNOWFLAKE_EPOCH, 1, 1));

// user struct

#[derive(SimpleObject)]
struct User {
    pub id: i64,
    pub username: String,

    /// the user's avatar url
    pub avatar: Option<String>,

    /**
     * 1 >> 0 = Staff
     * 1 >> 1 = Tester
     */
    pub flags: i32,
}

// graphql resolvers

#[derive(Default)]
pub struct UserQuery;

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserQuery {
    async fn api_version(&self) -> &'static str {
        "0.0.1"
    }

    /// get a user by their id
    async fn get_user<'ctx>(&self, ctx: &Context<'ctx>, id: String) -> User {
        let id = id.parse::<i64>().unwrap();
        let user = get_db(ctx)
            .query_one("SELECT * FROM users WHERE id = $1", &[&id])
            .await
            .unwrap();

        User {
            id,
            username: user.get("username"),
            avatar: user.get("avatar"),
            flags: user.get("flags"),
        }
    }
}

#[async_graphql::Object]
impl UserMutation {
    /// create an account on harmony
    async fn signup<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        email: String,
        password: String,
        username: String,
        avatar: Option<String>,
    ) -> Result<User, Error> {
        // hash the password
        let hash = bcrypt::hash(password)?;

        // generate an id for the user
        let id = unsafe { SNOWFLAKE.generate() };

        // insert the user into the database
        get_db(ctx)
            .execute(
                "INSERT INTO users (id, username, avatar, email, hash) VALUES ($1, $2, $3, $4, $5)",
                &[&id, &username, &avatar, &email, &hash],
            )
            .await
            .map_err(|err| {
                "Account already exists with that ".to_owned()
                    + (if err.to_string().contains("Key (email)") {
                        "email."
                    } else {
                        "username."
                    })
            })?;

        Ok(User {
            id,
            username,
            avatar,
            flags: 0,
        })
    }

    /// login to an account on harmony
    async fn login<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        email: String,
        password: String,
    ) -> Result<Option<String>, Error> {
        let db = get_db(ctx);
        let user = db
            .query_one("SELECT id, hash FROM users WHERE email = $1", &[&email])
            .await?;
        let hash = user.get("hash");
        let should_authenticate = bcrypt::verify(password, hash);

        if should_authenticate {
            #[derive(Serialize)]
            struct Claims {
                user_id: i64,
                expiration: i64,
            }
            let token = encode(
                &Header::new(Algorithm::HS512),
                &Claims {
                    user_id: user.get("id"),
                    expiration: Utc::now()
                        .checked_add_signed(chrono::Duration::hours(
                            dotenv!("JWT_EXPIRATION_HOURS").parse::<i64>().unwrap(),
                        ))
                        .expect("valid timestamp")
                        .timestamp(),
                },
                &EncodingKey::from_secret(dotenv!("JWT_SECRET").as_bytes()),
            )?;

            Ok(Some(token))
        } else {
            Ok(None)
        }
    }
}
