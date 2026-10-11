use opensovd_client::Client;
#[tokio::main]
async fn main() {
    let client = Client::builder().base_uri("http://127.0.0.1:20002/Vehicle/v15").unwrap().build().unwrap();
    let res = client.component("flxc1000").list_data().send().await;
    println!("{:?}", res);
}
