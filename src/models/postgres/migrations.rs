/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2023 Max Maisel

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
use diesel::{pg::PgConnection, Connection};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db");

pub fn run_migrations(db_url: &str) -> Result<(), String> {
    let mut conn =
        PgConnection::establish(db_url).map_err(|e| e.to_string())?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| e.to_string())?;

    Ok(())
}
