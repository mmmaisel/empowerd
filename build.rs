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
use std::process::Command;

const GUI_DIR: &str = "gui";

fn main() {
    // TODO: run conditionally when input files have changed
    Command::new("npm")
        .arg("install")
        .current_dir(GUI_DIR)
        .status()
        .unwrap();
    Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(GUI_DIR)
        .status()
        .unwrap();
}
