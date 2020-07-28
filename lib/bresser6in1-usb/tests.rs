use crate::Client;
use crate::Bresser6in1Buf;
use crate::Parser;
use crate::ParserResult;
use crate::Data;

use hidapi::HidApi;

use std::io::Cursor;

use bytes::Bytes;
use bytes::BytesMut;
use bytes::Buf;
use bytes::BufMut;

struct FakeReader {
    pos: usize
}

impl FakeReader {
    const FAKE_DATA: &'static [&'static [u8; 64]] = &[
        concat_bytes!(
            b"\xfe\0\0\0\03\x19- --.- -- --.- -- --.- --\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x91\x8f\xfd"
        ),
        concat_bytes!(
            b"\xfa\x03\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\xec\x85\xfd"
        ),
        concat_bytes!(
            b"\xfa\x03\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\xec\x85\xfd"
        ),
        concat_bytes!(
            b"\xfb\0\0\0\0A6&dateutc=now&baromin=30.03&tempf=42.8&hum",
            b"idity=60&wind\x1f\x98\xfd"
        ),
        concat_bytes!(
            b"\xfb\0\0\0\0B6speedmph=0&windgustmph=0&winddir=129&dewp",
            b"tf=29.8&raini\xbd\xe2\xfd"
        ),
        concat_bytes!(
            b"\xfb\0\0\0\0C6n=0&dailyrainin=0&UV=0&indoortempf=68.7&i",
            b"ndoorhumidity\xf6!\xfd"
        ),
        concat_bytes!(
            b"\xfb\0\0\0\0D\x03=49\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\xb4y\xfd"
        ),
        concat_bytes!(
            b"\xfa\x03\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\xec\x85\xfd"
        ),
        concat_bytes!(
            b"\xfe\0\0\0\0163 2020-01-17 17:30 20.4 49 6.0 60 0.0 0.0 ",
            b"0.0 0.0 129 F\x16\xfd"
        ),
        concat_bytes!(
            b"\xfe\0\0\0\026SE 1017 954 0 -1.2 --.- --.- -- --.- -- -",
            b"-.- -- --.- -\x9c$\xfd"
        ),
        concat_bytes!(
            b"\xfe\0\0\0\03\x19- --.- -- --.- -- --.- --\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x91\x8f\xfd"
        ),
        concat_bytes!(
            b"\xfa\x03\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\xec\x85\xfd"
        ),
        concat_bytes!(
            b"\xfa\x03\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\xec\x85\xfd"
        ),
        concat_bytes!(
            b"\xfe\0\0\0\0163 2020-01-17 17:30 20.4 49 6.0 61 0.0 0.0 ",
            b"0.0 0.0 129 +\xce\xfd"
        ),
        concat_bytes!(
            b"\xfe\0\0\0\026SE 1017 954 0 -1.0 --.- --.- -- --.- -- --",
            b".- -- --.- -d\x93\xfd"
        ),
        concat_bytes!(
            b"\xfe\0\0\0\03\x19- --.- -- --.- -- --.- --\0\0\0\0\0\0\0",
            b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x91\x8f\xfd"
        ),
    ];

    pub fn new() -> FakeReader {
        return FakeReader { pos: 0 };
    }

    pub fn read_data(&mut self, buf: &mut BytesMut) -> Result<(), String> {
        buf.put(&FakeReader::FAKE_DATA[self.pos][..]);
        println!(
            "Received 64 bytes: {}",
            String::from_utf8_lossy(FakeReader::FAKE_DATA[self.pos]
        ));
        self.pos += 1;
        if self.pos == FakeReader::FAKE_DATA.len() {
            self.pos = 0;
        }
        return Ok(());
    }
}

#[test]
fn parse_fake_data() {
    let mut client = FakeReader::new();

    let mut buf: BytesMut = BytesMut::with_capacity(64);
    let mut parser = Parser::new();
    let mut message_was_parsed = false;

    for _ in 0..12 {
        if let Err(e) = client.read_data(&mut buf)
        {
            panic!(e)
        };
        let mut cursor = Cursor::new(&mut buf);
        let msg = match cursor.to_message() {
            Ok(x) => x,
            Err(e) => panic!(e)
        };
        println!("Decoded: {:?}", msg);
        assert_eq!(0, cursor.remaining(), "Did not read whole buffer.");

        if let Ok(result) = parser.parse_message(msg) {
            if let ParserResult::Success(payload) = result {
                assert_eq!(concat!(
                    "3 2020-01-17 17:30 20.4 49 6.0 60 0.0 0.0 0.0 0.0 129 ",
                    "SE 1017 954 0 -1.2 --.- --.- -- --.- -- --.- -- --.- -",
                    "- --.- -- --.- -- --.- --"), payload);
                message_was_parsed = true;

                let data = Data::from_string(payload);
                match data {
                    Ok(x) => {
                        println!("Parsed data: {:?}", x);
                        assert_eq!(1579282200, x.timestamp);
                        assert_eq!(20.4, x.temperature_in);
                        assert_eq!(49, x.humidity_in);
                        assert_eq!(6.0, x.temperature_out);
                        assert_eq!(60, x.humidity_out);
                        assert_eq!(0.0, x.rain_day);
                        assert_eq!(0.0, x.rain_actual);
                        assert_eq!(0.0, x.wind_actual);
                        assert_eq!(0.0, x.wind_gust);
                        assert_eq!(129, x.wind_dir);
                        assert_eq!(1017, x.baro_sea);
                        assert_eq!(954, x.baro_absolute);
                        assert_eq!(0.0, x.uv_index);
                        assert_eq!(-1.2, x.dew_point);
                    },
                    Err(e) => panic!(e)
                }
            }
        };
        buf.clear();
    }
    assert!(message_was_parsed);
}

//#[test]
fn parse_usb_data() {
    let mut client = Client::new(None);

    match client.device_info() {
        Ok(x) => println!("{}", x),
        Err(e) => panic!(e),
    }

    println!("Reading data from USB...");
    match client.read_data() {
        Ok(x) => println!("done: {:?}", x),
        Err(e) => panic!(e),
    };
}

//#[test]
fn decode_data() {
    let data = vec![
        0xFA, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xEC, 0x85, 0xFD, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x31, 0x36, 0x31,
        0x20, 0x32, 0x30, 0x32, 0x30, 0x2D, 0x30, 0x31, 0x2D, 0x31, 0x30, 0x20,
        0x31, 0x34, 0x3A, 0x34, 0x32, 0x20, 0x32, 0x30, 0x2E, 0x39, 0x20, 0x35,
        0x32, 0x20, 0x31, 0x30, 0x2E, 0x36, 0x20, 0x35, 0x33, 0x20, 0x31, 0x2E,
        0x37, 0x37, 0x38, 0x20, 0x30, 0x2E, 0x30, 0x20, 0x35, 0x2E, 0x34, 0x20,
        0x37, 0x2E, 0x39, 0x20, 0x32, 0x49, 0x0B, 0xFD, 0xFE, 0x00, 0x00, 0x00,
        0x00, 0x32, 0x36, 0x38, 0x34, 0x20, 0x57, 0x4E, 0x57, 0x20, 0x31, 0x30,
        0x31, 0x39, 0x20, 0x39, 0x35, 0x36, 0x20, 0x30, 0x20, 0x31, 0x2E, 0x33,
        0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D,
        0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D,
        0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0xE0, 0x1C, 0xFD,
        0xFE, 0x00, 0x00, 0x00, 0x00, 0x33, 0x1C, 0x2D, 0x20, 0x2D, 0x2D, 0x20,
        0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D,
        0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x61, 0xE8, 0xFD,
    ];

    let data2 = vec![
        0xFE, 0x00, 0x00, 0x00, 0x00, 0x33, 0x1D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D,
        0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E,
        0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x43, 0x2F, 0xFD, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x31, 0x36, 0x31,
        0x20, 0x32, 0x30, 0x32, 0x30, 0x2D, 0x30, 0x31, 0x2D, 0x31, 0x30, 0x20,
        0x31, 0x34, 0x3A, 0x35, 0x37, 0x20, 0x32, 0x31, 0x2E, 0x31, 0x20, 0x35,
        0x32, 0x20, 0x31, 0x30, 0x2E, 0x35, 0x20, 0x35, 0x33, 0x20, 0x31, 0x2E,
        0x37, 0x37, 0x38, 0x20, 0x30, 0x2E, 0x30, 0x20, 0x31, 0x36, 0x2E, 0x35,
        0x20, 0x31, 0x36, 0x2E, 0x35, 0x66, 0xE9, 0xFD, 0xFE, 0x00, 0x00, 0x00,
        0x00, 0x32, 0x36, 0x20, 0x32, 0x32, 0x37, 0x20, 0x53, 0x57, 0x20, 0x31,
        0x30, 0x31, 0x39, 0x20, 0x39, 0x35, 0x36, 0x20, 0x30, 0x20, 0x31, 0x2E,
        0x32, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20,
        0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D,
        0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x06, 0x1D, 0xFD,
        0xFE, 0x00, 0x00, 0x00, 0x00, 0x33, 0x1D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D,
        0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E,
        0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x43, 0x2F, 0xFD,
    ];

    //@@þ1 6 1 2020-01-12 17:03 21.7 55 2.9 76 0.254 0.0 0.0 0.0 13 fý
    //@@þ2 6 5 SE 1023 960 0 -0.9 --.- --.- -- --.- -- --.- -- --.- ¯ÿý
    //@@þ3 . -- --.- -- --.- -- --.- -- wÛý

    //          date      time  t_in hum_in t_out hum_out  rain_d  rain_a  wind_a wind_g wind_d..
    //@@þ1 6 1 2020-01-12 17:03 21.7 55      2.9    76      0.254    0.0    0.0    0.0   13       fý
    //       ..dir wind baro_sea baro_abs  uv?  dew
    //@@þ2 6   5    SE     1023     960      0   -0.9 --.- --.- -- --.- -- --.- -- --.- ¯ÿý
    //@@þ3 . -- --.- -- --.- -- --.- -- wÛý

    //@@ûA 6 &dateutc=now&baromin=30.21&tempf=37.2&humidity=76&wind ".ý
    //@@ûB 6 speedmph=0&windgustmph=0&winddir=135&dewptf=30.3&raini ¶Æý
    //@@ûC 6 n=0&dailyrainin=0.01&UV=0&indoortempf=71.0&indoorhumid _0ý
    //@@ûD   ity=55ý

    //2020-01-12 13:24 21.1 52 5.6 65 0.0.0 2.5 2.5 16 S 1024 961 0 -0.5
    //--.- --.- -- --.- -- --.- -- --.- 3-- --.- -- --.- -- --.- --
    //&dateutc=now&baromin=30.24&tempf=42.0&humidity=65&windspeedmph=1.5
    //&windgustmph=1.5&winddir=169&dewptf=31.1&rainin=0&dailyrainin=0.01
    //&UV=0&indoortempf=69.9&indoorhumidity=52

    // dateutc=now        [actual]
    // &baromin=30.24     [inHG]    ==  1008 hPa    [hPa]  =  [inHg] /0.030
    // &temf=42.0         [F]       ==  5,5  °C     [C]    =  5/9 *([F] - 32)
    // &humidity=65       [%]
    // &windspeedmph=1.5  [mph]     ==  2.41 km/h   [km/h] =  [mph] * 0.44704
    // &windgustmph=1.5   [mph]     ==  2.41 km/h
    // &winddir=169       [deg]
    // &dewptf= 31.1      [F]       ==  -0,5 °C     [C]    =  5/9  * ([F] - 32)
    // &rainin=0.01       [Inch]    ==  0.25 mm     [mm]   =  [in] *  25.40
    // &UV= 0             [Index]
    // &indoortempf=69.9  [F]       ==  21,05°C
    // &indoorhumidity=52 [%]

    println!("data1: {}", String::from_utf8_lossy(&data));
    println!("data2: {}", String::from_utf8_lossy(&data2));

    println!("data len: {}", data.len());
    println!("data2 len: {}", data2.len());
}

//#[test]
fn read_raw_usb_data() {
    let api = match HidApi::new() {
        Ok(x) => x,
        Err(e) => panic!("Error initialising hidapi: {}", e),
    };

    let device = match api.open(0x1941, 0x8021) {
        Ok(x) => x,
        Err(e) => panic!("Error opening device: {}", e),
    };

    let mut buffer: [u8; 256] = [0; 256];

    for _ in 0..16 {
        let num_recv = match device.read(&mut buffer[..]) {
            Ok(x) => x,
            Err(e) => panic!("Error reading device: {}", e),
        };
        eprintln!("Received: {:?}", String::from_utf8_lossy(
            &buffer[..num_recv]));
    }
}
