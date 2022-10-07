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
use bytes::{Buf, BufMut, BytesMut};

pub struct SmaCmdIdentify {
    pub pkt_header: SmaPacketHeader,
    pub inv_header: SmaInvHeader,
    pub cmd: SmaCmdWord,
    pub _padding: [u8; 8],
    pub end: SmaEndToken,
}

impl SmaCmd for SmaCmdIdentify {
    fn serialize(&self, buffer: &mut BytesMut) {
        self.pkt_header.serialize(buffer);
        self.inv_header.serialize(buffer);
        self.cmd.serialize(buffer);
        buffer.put_slice(&self._padding);
        self.end.serialize(buffer);
    }

    fn opcode(&self) -> u32 {
        return self.cmd.opcode();
    }
}

impl SmaCmdIdentify {
    pub const OPCODE: u32 = 0x000002;
    pub const LENGTH: u16 = 12;

    pub fn new() -> SmaCmdIdentify {
        let mut retval = SmaCmdIdentify {
            pkt_header: SmaPacketHeader::new(
                SmaInvHeader::LENGTH + SmaCmdIdentify::LENGTH,
                SmaPacketHeader::SMA_PROTOCOL_INV,
            ),
            inv_header: SmaInvHeader::new(),
            cmd: SmaCmdWord::new(0, SmaCmdIdentify::OPCODE),
            _padding: [0; 8],
            end: SmaEndToken::new(),
        };
        retval
            .inv_header
            .infer_wordcount(retval.pkt_header.data_len);
        retval.inv_header.class = SmaInvHeader::CMD_CLASS_A0;
        return retval;
    }
}

pub struct SmaPayloadIdentify {
    _unknown: [u8; 48],
}

impl SmaPayloadIdentify {
    pub const LENGTH: usize = 48;

    pub fn deserialize(buffer: &mut dyn Buf) -> SmaPayloadIdentify {
        let mut unknown: [u8; 48] = [0; 48];
        buffer.copy_to_slice(&mut unknown);

        return SmaPayloadIdentify { _unknown: unknown };
    }

    pub fn validate(&self) -> Result<(), String> {
        // TODO
        return Ok(());
    }
}

pub struct SmaResponseIdentify {
    pub pkt_header: SmaPacketHeader,
    pub inv_header: SmaInvHeader,
    pub cmd: SmaCmdWord,
    pub payload: SmaPayloadIdentify,
    pub end: SmaEndToken,
}

impl SmaResponse for SmaResponseIdentify {
    fn extract_data(&self) -> SmaData {
        SmaData::Endpoint(SmaEndpoint {
            susy_id: self.inv_header.app.susy_id,
            serial: self.inv_header.app.serial,
            ctrl: self.inv_header.app.ctrl,
        })
    }

    fn get_header(&self) -> SmaHeader {
        SmaHeader::Inv(&self.inv_header)
    }

    fn opcode(&self) -> u32 {
        self.cmd.opcode()
    }

    fn validate(&self) -> Result<(), String> {
        self.pkt_header.validate()?;
        if self.pkt_header.data_len != SmaResponseIdentify::LENGTH {
            return Err("SmaResponseIdentify has invalid length".to_string());
        }
        self.inv_header.validate()?;
        self.cmd.validate()?;
        self.payload.validate()?;
        self.end.validate()?;
        return Ok(());
    }
}

impl SmaResponseIdentify {
    pub const LENGTH: u16 = 0x004E;
}
