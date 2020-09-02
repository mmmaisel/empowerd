#[derive(juniper::GraphQLObject)]
pub struct Valve {
    pub id: i32,
    pub open: bool,
    pub name: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct InputValve {
    pub id: i32,
    pub open: bool,
}
