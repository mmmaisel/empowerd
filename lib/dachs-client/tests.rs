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
