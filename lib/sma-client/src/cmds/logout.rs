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
    pub data_header: SmaDataHeader,
    pub cmd: SmaCmdWord,
    pub _padding: u32,
    pub end: SmaEndToken,
}

impl SmaCmd for SmaCmdLogout {
    fn serialize(&self, buffer: &mut BytesMut) {
        self.pkt_header.serialize(buffer);
        self.data_header.serialize(buffer);
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
                SmaDataHeader::LENGTH + SmaCmdLogout::LENGTH,
            ),
            data_header: SmaDataHeader::new(),
            cmd: SmaCmdWord::new(0x0E, SmaCmdLogout::OPCODE),
            _padding: 0xFFFFFFFF,
            end: SmaEndToken::new(),
        };
        retval
            .data_header
            .infer_wordcount(retval.pkt_header.data_len);
        retval.data_header.class = SmaDataHeader::CMD_CLASS_A0;
        retval.data_header.app.ctrl = 3;
        retval.data_header.dst.ctrl = 3;
        return retval;
    }

    #[allow(unused)]
    fn fragment_id(&self) -> u16 {
        return self.data_header.fragment_id;
    }

    #[allow(unused)]
    fn packet_id(&self) -> u16 {
        return self.data_header.packet_id;
    }

    #[allow(unused)]
    fn opcode(&self) -> u32 {
        return self.cmd.opcode();
    }
}

// This command has no response
