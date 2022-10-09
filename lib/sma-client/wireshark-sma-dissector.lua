--[[***************************************************************************]
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
[***************************************************************************--]]
sma_protocol = Proto("SMA",  "SMA Speedwire")

sma_magic = ProtoField.uint32("sma.magic", "SMA Magic", base.HEX)
hdr_len = ProtoField.uint16("sma.hdr_len", "Header Length", base.DEC)
hdr_magic = ProtoField.uint16("sma.hdr_magic", "Magic", base.HEX)
hdr_group = ProtoField.uint32("sma.hdr_group", "Group", base.DEC)
hdr_dlen = ProtoField.uint16("sma.hdr_dlen", "Data Length", base.DEC)
hdr_version = ProtoField.uint16("sma.hdr_version", "Version", base.DEC)
hdr_proto = ProtoField.uint16("sma.hdr_proto", "Protocol", base.HEX)

em_magic = ProtoField.uint16("sma.em_magic", "Magic", base.HEX)
em_serial = ProtoField.uint32("sma.em_serial", "Serial Number", base.DEC)
em_timestamp = ProtoField.uint32("sma.em_timestamp", "Timestamp ms", base.DEC)
obis_record = ProtoField.none("sma.obis_number", "Obis Record:")

sma_end = ProtoField.int16("sma.end", "End token", base.DEC)

sma_protocol.fields = {
  sma_magic,
  hdr_len, hdr_magic, hdr_group, hdr_dlen, hdr_version, hdr_proto,
  em_magic, em_serial, em_timestamp, obis_record,
  sma_end
}

function protocol_name(proto_id)
  if proto_id == 0x6069 then
    return "(Energy Meter)"
  elseif proto_id == 0x6065 then
    return "(Inverter)"
  else
    return "(Unknown)"
  end
end

function format_obis(obis_channel, obis_meas, obis_type, obis_tariff, val)
  local base_meas = obis_meas % 0x14
  local phase_meas = math.floor(obis_meas / 0x14)
  local unit = ""
  local class = ""

  if base_meas == 1 or base_meas == 2 then
    class = base_meas == 1 and "Active Power +" or "Active Power -"
    if obis_type == 4 then
      unit = "e-1 W"
    else
      unit = " Ws"
    end
  elseif base_meas == 3 or base_meas == 4 then
    class = base_meas == 3 and "Reactive Power +" or "Reactive Power -"
    if obis_type == 4 then
      unit = "e-1 VAr"
    else
      unit = " VArs"
    end
  elseif base_meas == 9 or base_meas == 10 then
    class = base_meas == 9 and "Apparent Power +" or "Apparent Power -"
    if obis_type == 4 then
      unit = "e-1 VA"
    else
      unit = " VAs"
    end
  elseif base_meas == 11 then
    class = "Current"
    unit = " mA"
  elseif base_meas == 12 then
    class = "Voltage"
    unit = " mV"
  elseif base_meas == 13 then
    class = "Power Factor"
    unit = "e-3"
  end

  if obis_channel == 0 and phase_meas == 0 then
    class = "Sum " .. class
  elseif obis_channel == 0 and phase_meas <= 3 then
    class = string.format("L%d ", phase_meas) .. class
  elseif obis_channel == 144 then
    class = "Version"
  else
    class = "Unknown"
  end

  return string.format(
    "%s (%d:%d.%d.%d): %s%s",
    class,
    obis_channel,
    obis_meas,
    obis_type,
    obis_tariff,
    val,
    unit
  )
end

function sma_protocol.dissector(buffer, pinfo, tree)
  length = buffer:len()
  if length == 0 then return end

  pinfo.cols.protocol = sma_protocol.name

  if buffer(0,4):uint() ~= 0x534d4100 then return end
  local dlen = buffer(12,2):uint();

  local smatree = tree:add(sma_protocol, buffer(0, dlen+20), "SMA Speedwire")
  local hdrtree = smatree:add(sma_protocol, buffer(0,18), "Header")

  hdrtree:add(sma_magic, buffer(0,4))
  hdrtree:add(hdr_len, buffer(4,2))
  hdrtree:add(hdr_magic, buffer(6,2))
  hdrtree:add(hdr_group, buffer(8,4))
  hdrtree:add(hdr_dlen, buffer(12,2))
  hdrtree:add(hdr_version, buffer(14,2))
  hdrtree:add(hdr_proto, buffer(16,2)):append_text(
    " " .. protocol_name(buffer(16,2):uint()))

  if buffer(16,2):uint() == 0x6069 then
    local emtree = smatree:add(sma_protocol, buffer(18,dlen-4), "Energy Meter")
    emtree:add(em_magic, buffer(18,2))
    emtree:add(em_serial, buffer(20,4))
    emtree:add(em_timestamp, buffer(24,4))

    local pos = 12;
    local datatree = emtree:add(sma_protocol, buffer(28,dlen-12), "Payload")

    while pos < dlen do
      local obis_channel = buffer(pos+16, 1):uint()
      local obis_meas = buffer(pos+17, 1):uint()
      local obis_type = buffer(pos+18, 1):uint()
      local obis_tariff = buffer(pos+19, 1):uint()
      local obis_data = 0
      local obis_len = 0

      if obis_type == 8 then
        obis_data = buffer(pos+20, 8):uint64()
        obis_len = 12
      else
        obis_data = buffer(pos+20, 4):uint()
        obis_len = 8
      end

      datatree:add(obis_record, buffer(pos+16, obis_len)):append_text(
        " " .. format_obis(
          obis_channel, obis_meas, obis_type, obis_tariff, obis_data
        )
      )
      pos = pos + obis_len
    end
  end

  local endtree = smatree:add(sma_protocol, buffer(dlen+16), "End token")
  endtree:add(sma_end, buffer(dlen+16, 4))
end

local udp_port = DissectorTable.get("udp.port")
udp_port:add(9522, sma_protocol)

