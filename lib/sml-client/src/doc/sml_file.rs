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
use super::sml_message::*;
use super::sml_stream::*;

// TODO: use byteorder, get rid of bytes, also in SMA
use bytes::Buf;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlFile {
    pub version: u32,
    pub messages: Vec<SmlMessage>,
}

impl SmlFile {
    pub fn deserialize_streams(
        streams: Vec<SmlStream>,
    ) -> Result<Vec<SmlFile>, String> {
        let mut parsed_files: Vec<SmlFile> = Vec::new();
        for stream in streams.into_iter() {
            parsed_files.push(SmlFile::deserialize(stream)?);
        }
        return Ok(parsed_files);
    }

    pub fn deserialize(stream: SmlStream) -> Result<SmlFile, String> {
        // TODO: validate len here
        let mut messages: Vec<SmlMessage> = Vec::new();
        let mut buffer = std::io::Cursor::new(stream.data);
        while buffer.remaining() > stream.padding {
            messages.push(SmlMessage::deserialize(&mut buffer)?);
        }

        return Ok(SmlFile {
            version: stream.version,
            messages: messages,
        });
    }
}
