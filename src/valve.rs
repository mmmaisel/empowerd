#[derive(juniper::GraphQLObject)]
pub struct Valve {
    pub id: i32,
    pub state: bool,
}
