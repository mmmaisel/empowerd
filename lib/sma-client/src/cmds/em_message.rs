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
use super::*;
use bytes::{Buf, BufMut, BytesMut};
use std::collections::BTreeMap;

pub struct SmaEmMessage {
    pub pkt_header: SmaPacketHeader,
    pub em_header: SmaEmHeader,
    pub payload: SmaEmPayload,
    pub end: SmaEndToken,
}

impl SmaEmMessage {
    pub fn new() -> Self {
        Self {
            pkt_header: SmaPacketHeader::new(
                SmaEmHeader::LENGTH,
                SmaPacketHeader::SMA_PROTOCOL_EM,
            ),
            em_header: SmaEmHeader::new(),
            payload: SmaEmPayload::new(),
            end: SmaEndToken::new(),
        }
    }

    pub fn serialize(&self, buffer: &mut BytesMut) {
        self.pkt_header.serialize(buffer);
        self.em_header.serialize(buffer);
        self.payload.serialize(buffer);
        self.end.serialize(buffer);
    }

    pub fn update_len(&mut self) {
        self.pkt_header.data_len =
            self.payload.payload_len() + SmaEmHeader::LENGTH + 2;
    }
}

impl SmaResponse for SmaEmMessage {
    fn extract_data(&self) -> SmaData {
        SmaData::EmPayload(self.payload.0.to_owned())
    }

    fn get_header(&self) -> SmaHeader {
        SmaHeader::Em(&self.em_header)
    }

    fn opcode(&self) -> u32 {
        0
    }

    fn validate(&self) -> Result<(), String> {
        self.pkt_header.validate()?;
        // TODO: validate length
        self.em_header.validate()?;
        self.payload.validate()?;
        self.end.validate()?;

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmaEmPayload(pub BTreeMap<u32, u64>);

impl SmaEmPayload {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn payload_len(&self) -> u16 {
        self.0.iter().fold(0, |acc, (obis, _)| {
            if *obis == 0x90000000 || *obis & 0xFF00 == 0x0400 {
                acc + 8
            } else if *obis & 0xFF00 == 0x0800 {
                acc + 12
            } else {
                acc
            }
        })
    }

    pub fn serialize(&self, buffer: &mut BytesMut) {
        for (obis, val) in &self.0 {
            buffer.put_u32(*obis);
            if *obis == 0x90000000 || *obis & 0xFF00 == 0x0400 {
                buffer.put_u32(*val as u32);
            } else if *obis & 0xFF00 == 0x0800 {
                buffer.put_u64(*val);
            }
        }
    }

    pub fn deserialize(buffer: &mut dyn Buf, mut len: u16) -> Self {
        let mut payload = Self::new();

        while len >= 8 {
            let obis = buffer.get_u32();
            let value = if obis == 0x90000000 || obis & 0xFF00 == 0x0400 {
                len -= 8;
                buffer.get_u32() as u64
            } else if obis & 0xFF00 == 0x0800 {
                len -= 12;
                buffer.get_u64()
            } else {
                0
            };
            payload.0.insert(obis, value);
        }

        payload
    }

    pub fn validate(&self) -> Result<(), String> {
        self.0.iter().try_for_each(|(obis, _)| {
            if *obis == 0x90000000
                || *obis & 0xFF00 == 0x0400
                || *obis & 0xFF00 == 0x0800
            {
                Ok(())
            } else {
                Err(format!("Found invalid obis number {:X}", obis))
            }
        })
    }
}
