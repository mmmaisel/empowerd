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
use std::fmt;
use std::str::Utf8Error;

pub struct Message {
    pub msgtype: u8,
    pub fragment: u8,
    pub length: usize,
    pub content: [u8; 54],
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message")
            .field("msgtype", &self.msgtype)
            .field("fragment", &self.fragment)
            .field("length", &self.length)
            .field("data", &String::from_utf8_lossy(&self.content[0..54]))
            .finish()
    }
}

impl Message {
    pub fn to_string(&self) -> Result<&str, Utf8Error> {
        return std::str::from_utf8(&self.content[0..self.length]);
    }
}
