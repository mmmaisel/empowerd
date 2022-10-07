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
use super::*;
use bytes::{BufMut, BytesMut};

pub struct SmaCmdLogout {
    pub pkt_header: SmaPacketHeader,
    pub inv_header: SmaInvHeader,
    pub cmd: SmaCmdWord,
    pub _padding: u32,
    pub end: SmaEndToken,
}

impl SmaCmd for SmaCmdLogout {
    fn serialize(&self, buffer: &mut BytesMut) {
        self.pkt_header.serialize(buffer);
        self.inv_header.serialize(buffer);
        self.cmd.serialize(buffer);
        buffer.put_u32_le(self._padding);
        self.end.serialize(buffer);
    }

    fn opcode(&self) -> u32 {
        return self.cmd.opcode();
    }
}

impl SmaCmdLogout {
    pub const OPCODE: u32 = 0xFFFD01;
    pub const LENGTH: u16 = 8;

    pub fn new() -> SmaCmdLogout {
        let mut retval = SmaCmdLogout {
            pkt_header: SmaPacketHeader::new(
                SmaInvHeader::LENGTH + SmaCmdLogout::LENGTH,
                SmaPacketHeader::SMA_PROTOCOL_INV,
            ),
            inv_header: SmaInvHeader::new(),
            cmd: SmaCmdWord::new(0x0E, SmaCmdLogout::OPCODE),
            _padding: 0xFFFFFFFF,
            end: SmaEndToken::new(),
        };
        retval
            .inv_header
            .infer_wordcount(retval.pkt_header.data_len);
        retval.inv_header.class = SmaInvHeader::CMD_CLASS_A0;
        retval.inv_header.app.ctrl = 3;
        retval.inv_header.dst.ctrl = 3;
        return retval;
    }
}

// This command has no response
