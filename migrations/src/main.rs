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
#![forbid(unsafe_code)]

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use libempowerd::settings::Settings;

mod mig11000_fix_heatpump_heat;
mod mig4000_convert_battery_charge;
mod mig9000_to_postgres;

use mig11000_fix_heatpump_heat::{mig11000_fix_heatpump_heat, Mig11000Args};
use mig4000_convert_battery_charge::{
    mig4000_convert_battery_charge, Mig4000Args,
};
use mig9000_to_postgres::{mig9000_to_postgres, Mig9000Args};

/// Common migration command line arguments.
#[derive(Debug, Parser)]
#[command(version)]
#[command(about = "Empowerd data migrations")]
struct Cli {
    #[command(subcommand)]
    migration: Migration,
    /// Config filename
    #[clap(short, default_value("/etc/empowerd/empowerd.conf"))]
    config_path: PathBuf,
}

#[derive(Clone, Debug, Subcommand)]
enum Migration {
    /// Convert battery charge
    #[clap(name = "4000_convert_battery_charge")]
    Mig4000(Mig4000Args),
    /// Migration to PostgreSQL
    #[clap(name = "9000_to_postgres")]
    Mig9000(Mig9000Args),
    /// Fixup heatpump heat data
    #[clap(name = "11000_fix_heatpump_heat")]
    Mig11000(Mig11000Args),
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let settings = Settings::load_from_file(&cli.config_path)?;

    match cli.migration {
        Migration::Mig4000(args) => {
            mig4000_convert_battery_charge(settings, args).await?
        }
        Migration::Mig9000(args) => mig9000_to_postgres(settings, args).await?,
        Migration::Mig11000(args) => {
            mig11000_fix_heatpump_heat(settings, args).await?
        }
    }

    Ok(())
}
