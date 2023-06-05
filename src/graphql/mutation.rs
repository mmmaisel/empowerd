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
use slog::warn;
use std::convert::TryInto;
use std::time::Duration;
use tokio::sync::oneshot;

use super::appliance::{Appliance, InputAppliance};
use super::available_power::{AvailablePower, InputAvailablePower};
use super::poweroff_timer::{InputPoweroffTimer, PoweroffTimer};
use super::switch::{InputSwitch, Switch};
use crate::processors::{ApplianceCmd, AvailablePowerCmd, PoweroffTimerCmd};
use crate::Context;

pub struct Mutation;

#[juniper::graphql_object(Context = Context)]
impl Mutation {
    /// Login into API with username and password. Creates a session token.
    /// Session tokens must be send as "Bearer" in the "authorization" header.
    async fn login(
        ctx: &Context,
        username: String,
        password: String,
    ) -> juniper::FieldResult<String> {
        if username == ctx.globals.username {
            match argon2::verify_encoded(
                &ctx.globals.hashed_pw,
                password.as_bytes(),
            ) {
                Ok(valid) => {
                    if valid {
                        return ctx.globals.session_manager.register().map_err(
                            |e| e.to_string(&ctx.globals.logger).into(),
                        );
                    }
                }
                Err(e) => {
                    warn!(ctx.globals.logger, "Verify password failed: {}", e)
                }
            }
        }
        return Err("Incorrect user or password!".into());
    }

    /// Logout and invalidate used session token.
    async fn logout(ctx: &Context) -> juniper::FieldResult<String> {
        return match ctx.globals.session_manager.destroy(&ctx.token) {
            Ok(()) => Ok("Logged out".into()),
            Err(e) => Err(e.to_string(&ctx.globals.logger).into()),
        };
    }

    /// Controls available power threshold.
    async fn set_available_power(
        ctx: &Context,
        input: InputAvailablePower,
    ) -> juniper::FieldResult<AvailablePower> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let id_u: usize = input
            .id
            .try_into()
            .map_err(|_| "'id' is invalid".to_string())?;

        let processor =
            match ctx.globals.processor_cmds.available_power.get(id_u) {
                Some(x) => x,
                None => {
                    return Err(format!(
                        "AvailablePowerProcessor with id {} does not exist",
                        input.id
                    )
                    .into())
                }
            };

        let (tx, rx) = oneshot::channel();
        let cmd = AvailablePowerCmd::SetThreshold {
            threshold: input.threshold,
            resp: tx,
        };

        processor
            .issue_command(&ctx.globals.logger, cmd, rx)
            .await?;

        Ok(AvailablePower {
            id: input.id,
            threshold: input.threshold,
            name: processor.name.clone(),
            power: 0.0,
        })
    }

    /// Controls an appliance.
    async fn set_appliance(
        ctx: &Context,
        input: InputAppliance,
    ) -> juniper::FieldResult<Appliance> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let id_u: usize = input
            .id
            .try_into()
            .map_err(|_| "'id' is invalid".to_string())?;

        let processor = match ctx.globals.processor_cmds.appliance.get(id_u) {
            Some(x) => x,
            None => {
                return Err(format!(
                    "ApplianceProcessor with id {} does not exist",
                    input.id
                )
                .into())
            }
        };

        let (tx, rx) = oneshot::channel();
        let cmd = ApplianceCmd::SetForceOnOff {
            force_on_off: input.force_on_off,
            resp: tx,
        };

        processor
            .issue_command(&ctx.globals.logger, cmd, rx)
            .await?;

        Ok(Appliance {
            id: input.id,
            force_on_off: input.force_on_off,
            name: processor.name.clone(),
        })
    }

    /// Controls a poweroff timer.
    async fn set_poweroff_timer(
        ctx: &Context,
        input: InputPoweroffTimer,
    ) -> juniper::FieldResult<PoweroffTimer> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let id_u: usize = input
            .id
            .try_into()
            .map_err(|_| "'id' is invalid".to_string())?;
        let on_time_u: u64 = input
            .on_time
            .try_into()
            .map_err(|_| "'on_time' is_invalid".to_string())?;

        if on_time_u == 0 {
            return Err(format!("'on_time' must be greater than zero!").into());
        }

        let on_time = Duration::from_secs(on_time_u);

        let processor =
            match ctx.globals.processor_cmds.poweroff_timer.get(id_u) {
                Some(x) => x,
                None => {
                    return Err(format!(
                        "PoweroffTimerProcessor with id {} does not exist",
                        input.id
                    )
                    .into())
                }
            };

        let switch_id = match processor.switch_id {
            Some(x) => x as i32,
            None => return Err("Missing switch ID".into()),
        };

        let (tx, rx) = oneshot::channel();
        let cmd = PoweroffTimerCmd::SetOnTime { on_time, resp: tx };

        processor
            .issue_command(&ctx.globals.logger, cmd, rx)
            .await?;

        Ok(PoweroffTimer {
            id: input.id,
            on_time: input.on_time,
            switch_id,
        })
    }

    /// Open or close a switch.
    async fn set_switch(
        ctx: &Context,
        switch: InputSwitch,
    ) -> juniper::FieldResult<Switch> {
        if let Err(e) = ctx.globals.session_manager.verify(&ctx.token) {
            return Err(e.to_string(&ctx.globals.logger).into());
        }

        let channel: usize = match switch.id.try_into() {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        let name = match ctx.globals.gpio_switch.get_name(channel) {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        let icon = match ctx.globals.gpio_switch.get_icon(channel) {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        if let Err(e) = ctx.globals.gpio_switch.set_open(channel, switch.open) {
            return Err(e.into());
        }

        return Ok(Switch {
            id: switch.id,
            open: switch.open,
            name,
            icon,
        });
    }
}
