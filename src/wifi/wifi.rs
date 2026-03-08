use crate::{mk_static, wifi::WifiPins};
use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, StackResources};
use esp_hal::rng::Rng;
use esp_radio::wifi::WifiController;

pub async fn spawn_wifi_tasks<'a>(spawner: &Spawner, pins: WifiPins) {
    let (controller, interfaces) = esp_radio::wifi::new(pins.wifi, Default::default()).unwrap();
    let rng = Rng::new();
    let random_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
    let dhcp_config = DhcpConfig::default();
    let config = embassy_net::Config::dhcpv4(dhcp_config);
    let resources = &mut *mk_static!(StackResources<24>, StackResources::<24>::new());
    let (stack, runner) = embassy_net::new(interfaces.station, config, resources, random_seed);

    spawner.spawn(connection_task(controller)).unwrap();
    // spawner.spawn(wifi_task(runner)).ok();

    // wait_for_connection(&stack).await;

    // spawner.spawn(message_task(stack)).unwrap();
    // spawner.spawn(time_task(stack)).unwrap();
    // spawner.spawn(weather_task(stack)).unwrap();
    // spawner.spawn(timetable_task(stack)).unwrap();
    // spawner.spawn(tick_task(stack)).unwrap();
}

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    info!("Start connection_task");
    loop {
        // match esp_radio::wifi::sta_state() {
        //     WifiStaState::Connected => {
        //         // wait until we're no longer connected
        //         controller.wait_for_event(WifiEvent::StaDisconnected).await;
        //         Timer::after(Duration::from_secs(5)).await
        //     }
        //     _ => {}
        // }
        // if !matches!(controller.is_started(), Ok(true)) {
        //     let client_config = ModeConfig::Client(
        //         ClientConfig::default()
        //             .with_ssid(WIFI_SSID.into())
        //             .with_password(WIFI_PASSWORD.into()),
        //     );
        //     controller.set_config(&client_config).unwrap();
        //     info!("WiFi: Starting");
        //     controller.start_async().await.unwrap();
        //     info!("WiFi: Started");
        // }

        // info!("Scan");
        // let scan_config = ScanConfig::default().with_max(10);
        // let result = controller
        //     .scan_with_config_async(scan_config)
        //     .await
        //     .unwrap();
        // for ap in result {
        //     info!("{:?}", ap);
        // }

        // info!("WiFi: Connecting to {}…", WIFI_SSID);
        // match controller.connect_async().await {
        //     Ok(_) => info!("WiFi: connected!"),
        //     Err(e) => {
        //         info!("WiFi: Failed to connect: {:?}", e);
        //         Timer::after(Duration::from_secs(5)).await
        //     }
        // }
    }
}
