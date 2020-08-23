use super::Context;
use super::Valve;

pub struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
    async fn set_valves(
        ctx: &Context,
        state: bool,
    ) -> juniper::FieldResult<Valve> {
        return Ok(Valve {
            id: 17,
            state: state,
        });
    }
}
