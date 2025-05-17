/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2025 Max Maisel

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
use slog::{error, trace};
use tokio::sync::oneshot;

use super::{
    appliance::Appliance, available_power::AvailablePower,
    load_control::LoadControl, poweroff_timer::PoweroffTimer, switch::Switch,
};
use crate::{
    processors::{
        ApplianceCmd, AvailablePowerCmd, LoadControlCmd, PoweroffTimerCmd,
    },
    Context,
};

pub struct Query;

#[juniper::graphql_object]
#[graphql(Context = Context)]
#[graphql(scalar = S: juniper::ScalarValue)]
impl Query {
    /// Get the currently available power systems.
    async fn available_powers<S: juniper::ScalarValue>(
        ctx: &Context,
        executor: &juniper::Executor<'_, '_, Context, S>,
    ) -> juniper::FieldResult<Vec<AvailablePower>> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let lookahead = executor.look_ahead().children();
        let get_power = lookahead.has_child("power");
        let get_threshold = lookahead.has_child("threshold");

        let mut result_vec = Vec::<AvailablePower>::new();
        for (i, processor) in ctx
            .globals
            .processor_cmds
            .available_power
            .iter()
            .enumerate()
        {
            let mut result =
                AvailablePower::new(i as i32, processor.name.clone());
            if get_threshold {
                let (tx, rx) = oneshot::channel();
                let cmd = AvailablePowerCmd::GetThreshold { resp: tx };
                result.threshold = processor
                    .issue_command(&ctx.globals.logger, cmd, rx)
                    .await?;
            }
            if get_power {
                let (tx, rx) = oneshot::channel();
                let cmd = AvailablePowerCmd::GetPower { resp: tx };
                result.power = processor
                    .issue_command(&ctx.globals.logger, cmd, rx)
                    .await?;
            }

            result_vec.push(result);
        }

        Ok(result_vec)
    }

    /// Get all appliances.
    async fn appliances<S: juniper::ScalarValue>(
        ctx: &Context,
        executor: &juniper::Executor<'_, '_, Context, S>,
    ) -> juniper::FieldResult<Vec<Appliance>> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let lookahead = executor.look_ahead().children();
        let get_force_on_off = lookahead.has_child("forceOnOff");

        let mut result_vec = Vec::<Appliance>::new();
        for (i, processor) in
            ctx.globals.processor_cmds.appliance.iter().enumerate()
        {
            let mut result = Appliance::new(i as i32, processor.name.clone());
            if get_force_on_off {
                let (tx, rx) = oneshot::channel();
                let cmd = ApplianceCmd::GetForceOnOff { resp: tx };
                result.force_on_off = processor
                    .issue_command(&ctx.globals.logger, cmd, rx)
                    .await?;
            }

            result_vec.push(result);
        }

        Ok(result_vec)
    }

    /// Get grid load control mode.
    async fn load_control<S: juniper::ScalarValue>(
        ctx: &Context,
        executor: &juniper::Executor<'_, '_, Context, S>,
    ) -> juniper::FieldResult<Option<LoadControl>> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let lookahead = executor.look_ahead().children();
        let get_charge_mode = lookahead.has_child("chargeMode");

        let mut result_opt = None;
        if let Some(load_ctrl) = &ctx.globals.processor_cmds.load_control {
            let mut result = LoadControl::new(load_ctrl.name.clone());
            if get_charge_mode {
                let (tx, rx) = oneshot::channel();
                let cmd = LoadControlCmd::GetChargeMode { resp: tx };
                result.charge_mode = load_ctrl
                    .issue_command(&ctx.globals.logger, cmd, rx)
                    .await?;
            }

            result_opt = Some(result);
        }

        Ok(result_opt)
    }

    /// Get all poweroff timers.
    async fn poweroff_timers<S: juniper::ScalarValue>(
        ctx: &Context,
        executor: &juniper::Executor<'_, '_, Context, S>,
    ) -> juniper::FieldResult<Vec<PoweroffTimer>> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let lookahead = executor.look_ahead().children();
        let get_on_time = lookahead.has_child("onTime");

        let mut result_vec = Vec::<PoweroffTimer>::new();
        for (i, processor) in
            ctx.globals.processor_cmds.poweroff_timer.iter().enumerate()
        {
            let switch_id = match processor.switch_id {
                Some(x) => x as i32,
                None => return Err("Missing switch ID".into()),
            };

            let mut result = PoweroffTimer::new(i as i32, switch_id);
            if get_on_time {
                let (tx, rx) = oneshot::channel();
                let cmd = PoweroffTimerCmd::GetOnTime { resp: tx };
                result.on_time = match processor
                    .issue_command(&ctx.globals.logger, cmd, rx)
                    .await?
                    .as_secs()
                    .try_into()
                {
                    Ok(x) => x,
                    Err(e) => return Err(e.into()),
                };
            }

            result_vec.push(result);
        }

        Ok(result_vec)
    }

    /// Get the current state of the switches.
    async fn switches<S: juniper::ScalarValue>(
        ctx: &Context,
        executor: &juniper::Executor<'_, '_, Context, S>,
    ) -> juniper::FieldResult<Vec<Switch>> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let lookahead = executor.look_ahead().children();
        let get_open = lookahead.has_child("open");
        let switch_mux = &ctx.globals.switch_mux;

        return if get_open {
            trace!(ctx.globals.logger, "Query Switches: get open");
            let mut switches = Vec::with_capacity(switch_mux.len());
            for idx in switch_mux.ids() {
                let name = switch_mux.name(idx)?;
                let icon = switch_mux.icon(idx)?;
                // XXX: skip switch on error, append error at the end
                let open = match switch_mux.read_val(idx).await {
                    Ok(x) => x,
                    Err(e) => {
                        error!(ctx.globals.logger, "{}", e);
                        continue;
                    }
                };

                switches.push(Switch {
                    id: idx as i32,
                    open,
                    name,
                    icon,
                });
            }
            Ok(switches)
        } else {
            trace!(ctx.globals.logger, "Query Switches: get basic");
            switch_mux
                .ids()
                .into_iter()
                .map(|idx| {
                    let name = switch_mux.name(idx)?;
                    let icon = switch_mux.icon(idx)?;

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

    /// Get backend config string for UI.
    pub async fn backend_config<S: juniper::ScalarValue>(
        ctx: &Context,
        _executor: &juniper::Executor<'_, '_, Context, S>,
    ) -> juniper::FieldResult<String> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        Ok(serde_json::to_string(&ctx.globals.uiconfig)?)
    }
}
