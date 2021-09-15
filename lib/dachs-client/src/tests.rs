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
use crate::DachsClient;

#[tokio::test]
async fn read_dachs_data() {
    let client =
        DachsClient::new("127.0.0.1".into(), "AAABBBCCCDDDEEE".into(), None);

    match client.get_total_energy().await {
        Ok(x) => println!("Total energy is: {}", x),
        Err(e) => panic!("Get total energy failed: {}", e),
    }

    match client.get_runtime().await {
        Ok(x) => println!("Runtime is: {}", x),
        Err(e) => panic!("Get runtime failed: {}", e),
    }
}
