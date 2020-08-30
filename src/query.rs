use super::Context;
use super::Valve;

pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
    async fn valves(ctx: &Context) -> juniper::FieldResult<Vec<Valve>> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        return match ctx.globals.water_switch.get_open() {
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
