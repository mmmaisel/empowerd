#[derive(juniper::GraphQLObject)]
/// Reads a physical IO channel.
pub struct Valve {
    /// References the channel.
    pub id: i32,
    /// Currently open or closed.
    pub open: bool,
    /// Name of the channel.
    pub name: String,
}

#[derive(juniper::GraphQLInputObject)]
/// Controls a physical IO channel.
pub struct InputValve {
    /// References the channel.
    pub id: i32,
    /// Should be opened or closed.
    pub open: bool,
}
