use std::convert::TryInto;

use super::Context;
use super::InputValve;
use super::Valve;

pub struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
    async fn set_valve(
        ctx: &Context,
        valve: InputValve,
    ) -> juniper::FieldResult<Valve> {
        let channel: usize = match valve.id.try_into() {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        if let Err(e) = ctx.water_switch.set_open(channel, valve.open) {
            return Err(e.into());
        }

        return Ok(Valve {
            id: valve.id,
            open: valve.open,
        });
    }
}
