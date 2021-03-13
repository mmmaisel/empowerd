extern crate bytes;
use bytes::{BufMut, BytesMut};

use super::*;

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

    fn fragment_id(&self) -> u16 {
        return self.data_header.fragment_id;
    }

    fn packet_id(&self) -> u16 {
        return self.data_header.packet_id;
    }

    fn opcode(&self) -> u32 {
        return self.cmd.opcode();
    }
}

// This command has no response
