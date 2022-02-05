/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2022 Max Maisel

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

use super::switch::Switch;
use crate::Context;

pub struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    /// Get the current state of the switches.
    async fn switches(ctx: &Context) -> juniper::FieldResult<Vec<Switch>> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let lookahead = executor.look_ahead();
        let get_open = lookahead.has_child("open");
        let gpio_switch = &ctx.globals.gpio_switch;

        return if get_open {
            trace!(ctx.globals.logger, "Query Switches: get open");
            gpio_switch
                .get_ids()
                .into_iter()
                .map(|idx| {
                    let name = gpio_switch.get_name(idx)?;
                    let icon = gpio_switch.get_icon(idx)?;
                    let open = gpio_switch.get_open(idx)?;

                    return Ok(Switch {
                        id: idx as i32,
                        open,
                        name,
                        icon,
                    });
                })
                .collect()
        } else {
            trace!(ctx.globals.logger, "Query Switches: get basic");
            gpio_switch
                .get_ids()
                .into_iter()
                .map(|idx| {
                    let name = gpio_switch.get_name(idx)?;
                    let icon = gpio_switch.get_icon(idx)?;

                    return Ok(Switch {
                        id: idx as i32,
                        open: false,
                        name,
                        icon,
                    });
                })
                .collect()
        };
    }
}
