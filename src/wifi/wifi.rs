use crate::{
    channels::{RANDOM, Random},
    constants::{WIFI_PASSWORD, WIFI_SSID},
    mk_static,
    wifi::{WifiPins, http::http_get},
};
use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use esp_radio::wifi::{Config, Interface, WifiController, sta::StationConfig};
use no_std_strings::{str_format, str16, str256};

pub fn spawn_wifi_tasks<'a>(spawner: &Spawner, pins: WifiPins) {
    let (controller, interfaces) = esp_radio::wifi::new(pins.wifi, Default::default()).unwrap();
    let rng = Rng::new();
    let random_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
    let dhcp_config = DhcpConfig::default();
    let config = embassy_net::Config::dhcpv4(dhcp_config);
    let resources = &mut *mk_static!(StackResources<24>, StackResources::new());
    let (stack, runner) = embassy_net::new(interfaces.station, config, resources, random_seed);

    spawner.spawn(connection_task(controller)).unwrap();
    spawner.spawn(runner_task(runner)).unwrap();

    let _stack = mk_static!(Stack, stack);
    spawner.spawn(connection_status_task(_stack)).unwrap();
    spawner.spawn(periodic_request_task(_stack)).unwrap();
}

#[embassy_executor::task]
async fn connection_status_task(stack: &'static Stack<'static>) {
    loop {
        info!("connection_status_task loop");
        stack.wait_config_up().await;
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            let octets = config.address.address().octets();
            let value = str_format!(
                str16,
                "{}.{}.{}.{}",
                octets[0],
                octets[1],
                octets[2],
                octets[3]
            );
            RANDOM
                .send(crate::channels::Random::IP { value: Some(value) })
                .await;
            stack.wait_config_down().await;
            RANDOM
                .send(crate::channels::Random::IP { value: None })
                .await;
        }
    }
}

#[embassy_executor::task]
async fn runner_task(mut runner: Runner<'static, Interface<'static>>) {
    info!("start wifi runner_task");
    runner.run().await
}

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    info!("start wifi connection_task");

    loop {
        match controller.wait_for_disconnect_async().await {
            Ok(info) => info!("Disconnected with reason: {:?}", info.reason),
            Err(error) => info!("Disconnected with error: {:?}", error),
        }

        let config = Config::Station(
            StationConfig::default()
                .with_ssid(WIFI_SSID)
                .with_password(WIFI_PASSWORD.into()),
        );
        controller.set_config(&config).unwrap();

        info!("connecting to {}…", WIFI_SSID);
        match controller.connect_async().await {
            Ok(_) => info!("wifi connected"),
            Err(e) => {
                info!("failed to connect: {:?}", e);
                Timer::after(Duration::from_secs(5)).await
            }
        }
    }
}

fn parse_time(body: &str256) -> str16 {
    let mut iter = body.split("\n").into_iter();
    let res = str_format!(
        str16,
        "{}:{}:{}",
        iter.next().unwrap(),
        iter.next().unwrap(),
        iter.next().unwrap()
    );

    res
}

#[embassy_executor::task]
async fn periodic_request_task(stack: &'static Stack<'static>) {
    loop {
        stack.wait_config_up().await;
        let res = http_get(stack, "http://timetable.app.amateurinmotion.com/now").await;
        match res {
            Ok(resp) => {
                let current = parse_time(&resp);
                RANDOM.send(Random::Time { current }).await;
            }
            Err(_) => info!("HTTP error"),
        }
        Timer::after(Duration::from_secs(60)).await;
    }
}
