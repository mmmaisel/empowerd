use super::Context;
use super::Valve;

pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
    async fn valves(ctx: &Context) -> juniper::FieldResult<Vec<Valve>> {
        return Ok(vec![Valve {
            id: 1,
            state: false,
        }]);
    }
}
