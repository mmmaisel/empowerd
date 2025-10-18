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
use std::collections::HashMap;

use diesel::{
    migration::{
        Migration, MigrationSource, MigrationVersion, Result as MigResult,
    },
    pg::{Pg, PgConnection},
    Connection,
};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db");

/// [PgConnection] wrapper that runs migrations in integer order
struct IntVersionedMigConn(PgConnection);

impl MigrationHarness<Pg> for IntVersionedMigConn {
    fn run_migration(
        &mut self,
        migration: &dyn Migration<Pg>,
    ) -> MigResult<MigrationVersion<'static>> {
        self.0.run_migration(migration)
    }

    fn revert_migration(
        &mut self,
        migration: &dyn Migration<Pg>,
    ) -> MigResult<MigrationVersion<'static>> {
        self.0.revert_migration(migration)
    }

    fn applied_migrations(
        &mut self,
    ) -> MigResult<Vec<MigrationVersion<'static>>> {
        self.0.applied_migrations()
    }

    // Modified from Diesel's default implementation
    fn pending_migrations<S: MigrationSource<Pg>>(
        &mut self,
        source: S,
    ) -> MigResult<Vec<Box<dyn Migration<Pg>>>> {
        let applied_versions = self.applied_migrations()?;
        let mut migrations = source
            .migrations()?
            .into_iter()
            .map(|m| (m.name().version().as_owned(), m))
            .collect::<HashMap<_, _>>();

        for applied_version in applied_versions {
            migrations.remove(&applied_version);
        }

        let mut migrations = migrations.into_values().collect::<Vec<_>>();
        migrations.sort_unstable_by(|l, r| {
            let ver_l = l
                .name()
                .to_string()
                .split("_")
                .next()
                .expect("Migration is prefixed with version followed by '_'")
                .parse::<usize>()
                .expect("Migration version is a single integer");
            let ver_r = r
                .name()
                .to_string()
                .split("_")
                .next()
                .expect("Migration is prefixed with version followed by '_'")
                .parse::<usize>()
                .expect("Migration version is a single integer");

            ver_l.partial_cmp(&ver_r).unwrap()
        });

        Ok(migrations)
    }
}

pub fn run_migrations(db_url: &str) -> Result<(), String> {
    let conn = PgConnection::establish(db_url).map_err(|e| e.to_string())?;

    IntVersionedMigConn(conn)
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| e.to_string())?;

    Ok(())
}
