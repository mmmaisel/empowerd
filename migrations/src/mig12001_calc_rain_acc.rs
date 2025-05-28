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
use chrono::{DateTime, Utc};
use clap::Args;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use libempowerd::{
    models::{
        postgres::{run_migrations, Weather},
        units::{micrometer, second, Length},
    },
    settings::{Settings, SourceType},
};

#[derive(Args, Clone, Debug)]
pub struct Mig12001Args {
    /// Timestamp of the first record to be fixed
    #[arg(short, long)]
    start_timestamp: i64,
}

pub async fn mig12001_calc_rain_acc(
    settings: Settings,
    args: Mig12001Args,
) -> Result<(), String> {
    let pg_url = format!(
        "postgres://{}:{}@{}/{}",
        settings.database.user,
        settings.database.password,
        settings.database.url,
        settings.database.name,
    );
    tokio::task::block_in_place(|| run_migrations(&pg_url)).unwrap();

    let pool_cfg =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new(pg_url);
    let mut pool =
        Pool::builder(pool_cfg).build().map_err(|e| e.to_string())?;

    for source in &settings.sources {
        let result = match &source.variant {
            SourceType::Bresser6in1(_) => {
                migrate_rain(
                    &mut pool,
                    DateTime::from_timestamp(args.start_timestamp, 0).unwrap(),
                    source.series_id,
                )
                .await
            }
            _ => Ok(()),
        };

        if let Err(e) = result {
            eprintln!(
                "Migration of series '{}' finished with error: {}",
                &source.name, e
            );
        }
    }

    Ok(())
}

async fn migrate_rain(
    pool: &mut Pool<AsyncPgConnection>,
    mut now: DateTime<Utc>,
    series_id: i32,
) -> Result<(), String> {
    let mut postgres = pool.get().await.map_err(|e| e.to_string())?;

    let mut last_record =
        Weather::batch(&mut postgres, series_id, now.naive_utc(), 1)
            .await?
            .remove(0);
    // Assume starting record as already migrated if rain_acc is not zero.
    if last_record.rain_acc == Length::new::<micrometer>(0.0) {
        last_record.rain_acc = last_record
            .rain_day
            .unwrap_or(Length::new::<micrometer>(0.0));
        last_record.save_changes(&mut postgres, series_id).await?;
    }

    now = DateTime::from_timestamp(last_record.time.get::<second>() as i64, 0)
        .unwrap();

    eprintln!("Fixing series {series_id}");
    loop {
        let batch =
            Weather::batch(&mut postgres, series_id, now.naive_utc(), 1000)
                .await?;
        if batch.is_empty() {
            break;
        }
        eprintln!("Fixing batch starting at {}", now.timestamp());

        for mut record in batch {
            let current_rain_day =
                record.rain_day.unwrap_or(Length::new::<micrometer>(0.0));
            let last_rain_day =
                last_record
                    .rain_day
                    .unwrap_or(Length::new::<micrometer>(0.0));
            let delta_rain_day = current_rain_day - last_rain_day;

            record.rain_acc = if delta_rain_day < Length::new::<micrometer>(0.0)
            {
                last_record.rain_acc + current_rain_day
            } else {
                last_record.rain_acc + delta_rain_day
            };

            record.save_changes(&mut postgres, series_id).await?;
            last_record = record;
        }
        now = DateTime::from_timestamp(
            last_record.time.get::<second>() as i64 + 1,
            0,
        )
        .unwrap();
    }

    Ok(())
}
