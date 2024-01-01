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
use diesel::result::Error as DieselError;

#[derive(Clone, Debug)]
pub enum Error {
    NotFound,
    Canceled(String),
    Bug(String),
    Temporary(String),
    System(String),
    InvalidInput(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Error> for String {
    fn from(input: Error) -> Self {
        input.to_string()
    }
}

impl From<DieselError> for Error {
    fn from(input: DieselError) -> Self {
        match input {
            DieselError::NotFound => Self::NotFound,
            _ => Self::Temporary(input.to_string()),
        }
    }
}
