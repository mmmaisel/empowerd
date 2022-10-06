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

pub struct SmaCmdGetDayData {
    pub pkt_header: SmaPacketHeader,
    pub inv_header: SmaInvHeader,
    pub cmd: SmaCmdWord,
    pub start_time: u32,
    pub end_time: u32,
    pub end: SmaEndToken,
}

impl SmaCmd for SmaCmdGetDayData {
    fn serialize(&self, buffer: &mut BytesMut) {
        self.pkt_header.serialize(buffer);
        self.inv_header.serialize(buffer);
        self.cmd.serialize(buffer);
        buffer.put_u32_le(self.start_time);
        buffer.put_u32_le(self.end_time);
        self.end.serialize(buffer);
    }

    fn opcode(&self) -> u32 {
        return self.cmd.opcode();
    }
}

impl SmaCmdGetDayData {
    pub const OPCODE: u32 = 0x700002;
    pub const LENGTH: u16 = 12;

    pub fn new() -> SmaCmdGetDayData {
        let mut retval = SmaCmdGetDayData {
            pkt_header: SmaPacketHeader::new(
                SmaInvHeader::LENGTH + SmaCmdGetDayData::LENGTH,
                SmaPacketHeader::SMA_PROTOCOL_INV,
            ),
            inv_header: SmaInvHeader::new(),
            cmd: SmaCmdWord::new(0, SmaCmdGetDayData::OPCODE),
            start_time: 0,
            end_time: 0,
            end: SmaEndToken::new(),
        };
        retval
            .inv_header
            .infer_wordcount(retval.pkt_header.data_len);
        retval.inv_header.class = SmaInvHeader::CMD_CLASS_E0;
        return retval;
    }
}

#[derive(Clone, Copy)]
pub struct SmaDayDataRecord {
    timestamp: u32,
    energy: u32,
    _padding: u32,
}

impl SmaDayDataRecord {
    fn new() -> SmaDayDataRecord {
        return SmaDayDataRecord {
            timestamp: 0,
            energy: 0,
            _padding: 0,
        };
    }

    fn deserialize(buffer: &mut dyn Buf) -> SmaDayDataRecord {
        let timestamp = buffer.get_u32_le();
        let energy = buffer.get_u32_le();
        if buffer.remaining() > 4 {
            let _padding = buffer.get_u32_le();
        }

        return SmaDayDataRecord {
            timestamp: timestamp,
            energy: energy,
            _padding: 0,
        };
    }
}

pub struct SmaPayloadGetDayData {
    _padding: [u8; 8],
    data: [SmaDayDataRecord; SmaPayloadGetDayData::MAX_RECORD_COUNT],
}

impl SmaPayloadGetDayData {
    const MAX_RECORD_COUNT: usize = 117;
    const RECORD_LENGTH: u16 = 12;
    pub const MIN_LENGTH: usize = 8;

    // TODO: handle 0xFFFFFFFF values (NaN)
    pub fn deserialize(
        mut buffer: &mut dyn Buf,
        length: u16,
    ) -> SmaPayloadGetDayData {
        let mut padding: [u8; 8] = [0; 8];
        buffer.copy_to_slice(&mut padding);

        let mut records =
            [SmaDayDataRecord::new(); SmaPayloadGetDayData::MAX_RECORD_COUNT];

        for record in
            records[0..SmaResponseGetDayData::record_count(length)].iter_mut()
        {
            *record = SmaDayDataRecord::deserialize(&mut buffer);
        }
        return SmaPayloadGetDayData {
            _padding: padding,
            data: records,
        };
    }

    pub fn validate(&self) -> Result<(), String> {
        // TODO
        return Ok(());
    }
}

pub struct SmaResponseGetDayData {
    pub pkt_header: SmaPacketHeader,
    pub inv_header: SmaInvHeader,
    pub cmd: SmaCmdWord,
    pub payload: SmaPayloadGetDayData,
    pub end: SmaEndToken,
}

impl SmaResponse for SmaResponseGetDayData {
    fn extract_data(&self) -> SmaData {
        let mut datavec: Vec<TimestampedInt> =
            Vec::with_capacity(SmaPayloadGetDayData::MAX_RECORD_COUNT);

        for record in self.payload.data
            [0..SmaResponseGetDayData::record_count(self.pkt_header.data_len)]
            .iter()
        {
            datavec.push(TimestampedInt {
                timestamp: record.timestamp,
                value: record.energy,
            });
        }

        return SmaData::IntTimeSeries(datavec);
    }

    fn validate(&self) -> Result<(), String> {
        self.pkt_header.validate()?;
        // TODO: validate length
        /*        if self.pkt_header.data_len != SmaResponseIdentify::LENGTH
        {
            return Err("SmaResponseIdentify has invalid length");
        }*/
        self.inv_header.validate()?;
        self.cmd.validate()?;
        self.payload.validate()?;
        self.end.validate()?;
        return Ok(());
    }

    fn fragment_id(&self) -> u16 {
        return self.inv_header.fragment_id;
    }

    fn packet_id(&self) -> u16 {
        return self.inv_header.packet_id;
    }

    fn opcode(&self) -> u32 {
        return self.cmd.opcode();
    }
}

impl SmaResponseGetDayData {
    #[allow(unused)]
    pub const LENGTH: u16 = 0x004E;

    fn record_count(data_len: u16) -> usize {
        let payload_len = data_len - SmaInvHeader::LENGTH - 2;
        return ((payload_len - 8) / SmaPayloadGetDayData::RECORD_LENGTH)
            as usize;
    }
}
