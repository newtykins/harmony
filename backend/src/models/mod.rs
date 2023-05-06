mod user;

use async_graphql::{Context, EmptySubscription, Error, Object};
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use pwhash::bcrypt;
use serde::Serialize;
use user::User;

fn get_db<'t>(ctx: &Context<'t>) -> &'t tokio_postgres::Client {
    ctx.data::<tokio_postgres::Client>().unwrap()
}

pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    async fn api_version(&self) -> &'static str {
        "1.0"
    }

    async fn get_user<'ctx>(&self, ctx: &Context<'ctx>, id: String) -> User {
        let id = id.parse::<i64>().unwrap();
        let user = get_db(ctx)
            .query_one("SELECT * FROM users WHERE id = $1", &[&id])
            .await
            .unwrap();

        User {
            id: user.get("snowflake"),
            username: user.get("username"),
            avatar: user.get("avatar"),
            flags: user.get("flags"),
        }
    }
}

#[Object]
impl MutationRoot {
    async fn signup<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        username: String,
        email: String,
        password: String,
        avatar: String,
    ) -> Result<User, Error> {
        // hash the password
        let hash = bcrypt::hash(password)?;

        // generate an id for the user
        let id = User::generate_snowflake();

        // insert the user into the database
        get_db(ctx).execute("INSERT INTO users (id, username, avatar, about, email, hash) VALUES ($1, $2, $3, $4, $5, $6)", &[&id, &username, &avatar, &"", &email, &hash]).await.map_err(|err| "Account already exists with that ".to_owned() + (if err.to_string().contains("Key (email)") {"email."} else {"username."}))?;

        Ok(User {
            id,
            username,
            avatar,
            flags: 0,
        })
    }

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

pub type Schema = async_graphql::Schema<QueryRoot, MutationRoot, EmptySubscription>;
