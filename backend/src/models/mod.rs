mod user;

use async_graphql::{EmptySubscription, MergedObject};
use user::{UserMutation, UserQuery};

#[derive(MergedObject, Default)]
pub struct Query(UserQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(UserMutation);

pub type Schema = async_graphql::Schema<Query, Mutation, EmptySubscription>;
