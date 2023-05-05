mod user;

use async_graphql::{Context, EmptySubscription, Error, Object};
use pwhash::bcrypt;
use user::User;

fn get_db<'t>(ctx: &Context<'t>) -> &'t tokio_postgres::Client {
    ctx.data::<tokio_postgres::Client>().unwrap()
}

// Sun Jan 01 2023 00:00:00 GMT+0000 (Greenwich Mean Time)
const SNOWFLAKE_EPOCH: i64 = 1672531200;

pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    async fn api_version(&self) -> &'static str {
        "1.0"
    }

    async fn get_user<'ctx>(&self, ctx: &Context<'ctx>, snowflake: String) -> User {
        let snowflake = snowflake.parse::<i64>().unwrap();
        let user = get_db(ctx).query_one("SELECT * FROM users WHERE snowflake = $1", &[&snowflake]).await.unwrap();

        User {
            snowflake: user.get("snowflake"),
            username: user.get("username"),
            avatar: user.get("avatar")
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

        // generate a snowflake
        let mut snowflake_generator = rustflake::Snowflake::default();
        snowflake_generator.epoch(SNOWFLAKE_EPOCH);
        let snowflake = snowflake_generator.generate();

        // insert the user into the database
        get_db(ctx).execute("INSERT INTO users (snowflake, username, avatar, about, email, hash) VALUES ($1, $2, $3, $4, $5, $6)", &[&snowflake, &username, &avatar, &"", &email, &hash]).await.map_err(|err| "Account already exists with that ".to_owned() + (if err.to_string().contains("Key (email)") {"email."} else {"username."}))?;

        Ok(User {
            snowflake,
            username,
            avatar
        })
    }
}

pub type Schema = async_graphql::Schema<QueryRoot, MutationRoot, EmptySubscription>;
