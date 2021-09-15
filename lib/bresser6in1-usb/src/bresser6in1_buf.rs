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
use bytes::Buf;

use super::message::*;

pub trait Bresser6in1Buf: Buf {
    // One Msg:  64 Bytes
    // MSG_Type: 0xFE (254)
    // Padding: 4 x 0x00
    // Msg-Fragment: 1, 2, 3 (0x30 - 0x33)
    // Msg-Length 0x36 == 54
    // Data: 54 Bytes
    // Checksum 2 Bytes
    // Msg-End: 0xFD
    fn to_message(&mut self) -> Result<Message, String> {
        if self.remaining() != 64 {
            return Err(format!("Invalid message size: {}", self.remaining()));
        }

        let msgtype: u8 = self.get_u8();
        let _padding: u32 = self.get_u32_le();
        let fragment: u8 = self.get_u8();
        let length: usize = self.get_u8() as usize;
        let mut data: [u8; 54] = [0; 54];
        self.copy_to_slice(&mut data);
        let _crc: u16 = self.get_u16_le();
        let end: u8 = self.get_u8();

        // TODO: validate crc?

        if end != 0xFD {
            return Err(format!("Invalid EOF value {:X} received", end));
        }

        return Ok(Message {
            msgtype: msgtype,
            fragment: fragment,
            length: length,
            content: data,
        });
    }
}

impl<'a, T: Bresser6in1Buf + ?Sized> Bresser6in1Buf for &'a mut T {}
impl<T: AsRef<[u8]>> Bresser6in1Buf for std::io::Cursor<T> {}
