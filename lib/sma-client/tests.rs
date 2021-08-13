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
use crate::SmaClient;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn read_solar_data() {
    let sma_addr = match SmaClient::sma_sock_addr("192.168.5.75".to_string()) {
        Ok(x) => x,
        Err(e) => panic!(e),
    };
    let mut sma_client = SmaClient::new(None);

    let session = match sma_client.open().await {
        Ok(x) => x,
        Err(e) => panic!(e),
    };
    let identity = match sma_client.identify(&session, sma_addr).await {
        Err(e) => panic!("Could not identify SMA device, {}", e),
        Ok(x) => x,
    };

    eprintln!(
        "{} is {:X}, {:X}",
        sma_addr, identity.susy_id, identity.serial
    );
    sma_client.set_dst(sma_addr, identity.susy_id, identity.serial);

    if let Err(e) = sma_client.logout(&session).await {
        panic!("Logout failed: {}", e);
    }
    if let Err(e) = sma_client.login(&session, &"0000".to_string()).await {
        panic!("Login failed: {}", e);
    }

    let to = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(x) => x.as_secs() as u32,
        Err(e) => panic!(e),
    };
    let from = to - 3600;

    eprintln!("GetDayData from {} to {}", from, to);
    let data = match sma_client.get_day_data(&session, from, to).await {
        Err(e) => panic!("Get Day Data failed: {}", e),
        Ok(x) => x,
    };

    eprintln!("Get Day data returned {:?}", data);
    if let Err(e) = sma_client.logout(&session).await {
        panic!("Logout failed: {}", e);
    }
}
