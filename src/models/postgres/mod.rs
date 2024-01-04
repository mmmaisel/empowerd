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
use super::units;

pub mod battery;
pub mod bidir_meter;
pub mod generator;
pub mod heatpump;
pub mod simple_meter;
pub mod weather;

mod migrations;
mod schema;

pub use battery::Battery;
pub use bidir_meter::BidirMeter;
pub use generator::Generator;
pub use heatpump::Heatpump;
pub use migrations::run_migrations;
pub use simple_meter::SimpleMeter;
pub use weather::Weather;

macro_rules! impl_timeseries {
    ($raw_ty: ident, $ty: ident, $schema: ident) => {
        impl $raw_ty {
            pub fn query_last(
                series_id: i32,
            ) -> diesel::dsl::Order<
                diesel::dsl::Filter<
                    schema::$schema::table,
                    diesel::dsl::Eq<schema::$schema::series_id, i32>,
                >,
                diesel::dsl::Desc<schema::$schema::time>,
            > {
                use diesel::QueryDsl;
                schema::$schema::table
                    .filter(schema::$schema::series_id.eq(series_id))
                    .order(schema::$schema::time.desc())
            }
        }

        impl $ty {
            pub async fn last(
                conn: &mut diesel_async::AsyncPgConnection,
                series_id: i32,
            ) -> Result<$ty, crate::Error> {
                use diesel_async::RunQueryDsl;
                $raw_ty::query_last(series_id)
                    .first::<$raw_ty>(conn)
                    .await
                    .map(|x| x.into())
                    .map_err(|e| e.into())
            }

            pub async fn insert(
                &self,
                conn: &mut diesel_async::AsyncPgConnection,
                series_id: i32,
            ) -> Result<(), crate::Error> {
                use diesel_async::RunQueryDsl;
                let mut raw = $raw_ty::try_from(self)?;
                raw.series_id = series_id;

                diesel::insert_into(schema::$schema::table)
                    .values(&raw)
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        Error::Temporary(format!(
                            "Inserting record into series {series_id} failed: {e}",
                        ))
                    })?;

                Ok(())
            }

            pub async fn insert_bulk(
                data: Vec<Self>,
                conn: &mut diesel_async::AsyncPgConnection,
                series_id: i32,
            ) -> Result<usize, crate::Error> {
                use diesel_async::RunQueryDsl;
                let raw = data.iter().map(|x| {
                    let mut y = $raw_ty::try_from(x)?;
                    y.series_id = series_id;
                    Ok(y)
                }).collect::<Result<Vec<$raw_ty>, crate::Error>>()?;

                diesel::insert_into(schema::$schema::table)
                    .values(&raw)
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        Error::Temporary(format!(
                            "Inserting record into series {series_id} failed: {e}",
                        ))
                    })
            }
        }
    };
}

pub(crate) use impl_timeseries;
