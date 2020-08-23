#[derive(juniper::GraphQLObject)]
pub struct Valve {
    pub id: i32,
    pub open: bool,
}

#[derive(juniper::GraphQLInputObject)]
pub struct InputValve {
    pub id: i32,
    pub open: bool,
}
