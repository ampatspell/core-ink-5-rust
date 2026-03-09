use core::str::from_utf8;

use defmt::info;
use embassy_net::{
    Stack,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use no_std_strings::str256;
use reqwless::client::HttpClient;

#[derive(Debug, Clone)]
pub struct RequestFailedError;

pub async fn http_get(stack: &Stack<'static>, url: &str) -> Result<str256, RequestFailedError> {
    let dns = DnsSocket::new(*stack);
    let tcp_state = TcpClientState::<1, 4096, 4096>::new();
    let tcp = TcpClient::new(*stack, &tcp_state);

    info!("GET {}", url);

    let mut client = HttpClient::new(&tcp, &dns);
    let mut buffer = [0u8; 4096];

    let http_req = client.request(reqwless::request::Method::GET, url).await;

    if http_req.is_err() {
        info!("HTTP request: {:?}", http_req.err());
        return Err(RequestFailedError);
    }

    let mut http_req = http_req.unwrap();

    let response = http_req.send(&mut buffer).await;
    if response.is_err() {
        info!("HTTP request send");
        return Err(RequestFailedError);
    }
    let response = response.unwrap();

    if response.status.is_successful() {
        let body = response.body().read_to_end().await;
        if body.is_err() {
            info!("HTTP request read body {:?}", body.err());
            return Err(RequestFailedError);
        }
        let body = body.unwrap();

        let utf8 = from_utf8(body);
        if utf8.is_err() {
            info!("HTTP body utf8");
            return Err(RequestFailedError);
        }
        let utf8 = utf8.unwrap();

        let content = str256::from(utf8);

        info!("Ok: {}\n{}", url, utf8);

        return Ok(content);
    }

    Err(RequestFailedError)
}
