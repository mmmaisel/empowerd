use super::Context;
use super::Valve;

pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
    async fn valves(ctx: &Context) -> juniper::FieldResult<Vec<Valve>> {
        return match ctx.water_switch.get_open() {
            Ok(x) => x
                .iter()
                .enumerate()
                .map(|(idx, val)| {
                    return Ok(Valve {
                        id: idx as i32,
                        open: *val,
                    });
                })
                .collect(),
            Err(e) => Err(e.into()),
        };
    }
}
