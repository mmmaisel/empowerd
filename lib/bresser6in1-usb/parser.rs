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
use super::message::Message;

pub enum ParserResult {
    CollectingData,
    Success(String),
}

pub enum ParserError {
    IgnoredMessage(String),
    Error(String),
}

pub struct Parser {
    buffer: String,
    last_fragment: u8,
}

impl Parser {
    pub fn new() -> Parser {
        return Parser {
            buffer: String::with_capacity(162),
            last_fragment: 0,
        };
    }

    pub fn parse_message(
        &mut self,
        message: Message,
    ) -> Result<ParserResult, ParserError> {
        if message.msgtype != 0xFE {
            return Err(ParserError::IgnoredMessage(format!(
                "Received unknown message type {}.",
                message.msgtype
            )));
        }

        match message.fragment as char {
            '1' => {
                self.buffer.clear();
                self.append_msg(message)?;
                self.last_fragment = b'1';
                return Ok(ParserResult::CollectingData);
            }
            '2' => {
                if self.last_fragment != b'1' {
                    return Err(ParserError::IgnoredMessage(
                        "Wrong fragment order".to_string(),
                    ));
                }
                self.append_msg(message)?;
                self.last_fragment = b'2';
                return Ok(ParserResult::CollectingData);
            }
            '3' => {
                if self.last_fragment != b'2' {
                    return Err(ParserError::IgnoredMessage(
                        "Wrong fragment order".to_string(),
                    ));
                }
                self.append_msg(message)?;
                return Ok(ParserResult::Success(self.buffer.clone()));
            }
            _ => {
                return Err(ParserError::IgnoredMessage(format!(
                    "Received unknown fragment type {}.",
                    message.fragment
                )));
            }
        }
    }

    fn append_msg(&mut self, message: Message) -> Result<(), ParserError> {
        return match message.to_string() {
            Ok(x) => {
                self.buffer.push_str(x);
                Ok(())
            }
            Err(e) => Err(ParserError::Error(e.to_string())),
        };
    }
}
