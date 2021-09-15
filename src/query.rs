/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
use juniper::LookAheadMethods;
use slog::trace;

use super::Context;
use super::Valve;

pub struct Query;

#[juniper::graphql_object(Context = Context)]
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
