use super::Context;
use super::Valve;

pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
    async fn valves(ctx: &Context) -> juniper::FieldResult<Vec<Valve>> {
        // TODO: query real pins
        return Ok(vec![Valve { id: 1, open: false }]);
    }
}
