use std::convert::TryInto;

use super::Context;
use super::InputValve;
use super::Valve;

pub struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
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
                        return ctx
                            .globals
                            .session_manager
                            .register()
                            .map_err(|e| e.into());
                    }
                }
                Err(e) => println!("Verify password failed: {}", e),
            }
        }
        return Err("Incorrect user or password!".into());
    }

    async fn logout(ctx: &Context) -> juniper::FieldResult<String> {
        return match ctx.globals.session_manager.destroy(&ctx.token) {
            Ok(()) => Ok("Logged out".into()),
            Err(e) => Err(e.into()),
        };
    }

    async fn set_valve(
        ctx: &Context,
        valve: InputValve,
    ) -> juniper::FieldResult<Valve> {
        ctx.globals.session_manager.verify(&ctx.token)?;

        let channel: usize = match valve.id.try_into() {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        if let Err(e) = ctx.globals.water_switch.set_open(channel, valve.open) {
            return Err(e.into());
        }

        return Ok(Valve {
            id: valve.id,
            open: valve.open,
        });
    }
}
