use juniper::LookAheadMethods;
use slog::trace;

use super::Context;
use super::Valve;

pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
    /// Get the current state of the valves.
    async fn valves(ctx: &Context) -> juniper::FieldResult<Vec<Valve>> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let lookahead = executor.look_ahead();
        let get_open = lookahead.has_child("open");
        let get_name = lookahead.has_child("name");

        return if get_open && get_name {
            trace!(ctx.globals.logger, "Query Valves: get open and name");
            match ctx.globals.water_switch.get_open() {
                Ok(x) => x
                    .iter()
                    .enumerate()
                    .zip(ctx.globals.water_switch.get_names())
                    .map(|((idx, val), name)| {
                        return Ok(Valve {
                            id: idx as i32,
                            open: *val,
                            name: name,
                        });
                    })
                    .collect(),
                Err(e) => Err(e.into()),
            }
        } else if get_open {
            trace!(ctx.globals.logger, "Query Valves: get open");
            match ctx.globals.water_switch.get_open() {
                Ok(x) => x
                    .iter()
                    .enumerate()
                    .map(|(idx, val)| {
                        return Ok(Valve {
                            id: idx as i32,
                            open: *val,
                            name: "".into(),
                        });
                    })
                    .collect(),
                Err(e) => Err(e.into()),
            }
        } else {
            trace!(ctx.globals.logger, "Query Valves: get name");
            ctx.globals
                .water_switch
                .get_names()
                .into_iter()
                .enumerate()
                .map(|(idx, name)| {
                    return Ok(Valve {
                        id: idx as i32,
                        open: false,
                        name: name,
                    });
                })
                .collect()
        };
    }
}
