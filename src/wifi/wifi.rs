use crate::wifi::WifiPins;
use embassy_executor::Spawner;
use esp_radio::Controller;
use static_cell::StaticCell;

pub async fn spawn_wifi_tasks<'a>(spawner: &Spawner, pins: WifiPins) {
    let radio_init = {
        static CELL: StaticCell<Controller<'static>> = StaticCell::new();
        CELL.init(esp_radio::init().expect("WiFi: Failed to initialize Wi-Fi/BLE controller"))
    };

    esp_radio::init();

    // let (wifi_controller, wifi_interfaces) =
    //     esp_radio::wifi::new(radio_init, wifi, Default::default())
    //         .expect("WiFi: Failed to initialize Wi-Fi controller");

    // let rng = Rng::new();
    // let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    // let dhcp_config = DhcpConfig::default();

    // let config = embassy_net::Config::dhcpv4(dhcp_config);
    // let resources = {
    //     // max sockets
    //     static CELL: StaticCell<StackResources<24>> = StaticCell::new();
    //     CELL.init(StackResources::<24>::new())
    // };

    // let (stack, runner) = embassy_net::new(wifi_interfaces.sta, config, resources, net_seed);

    // spawner.spawn(connection_task(wifi_controller)).ok();
    // spawner.spawn(wifi_task(runner)).ok();

    // wait_for_connection(&stack).await;

    // spawner.spawn(message_task(stack)).unwrap();
    // spawner.spawn(time_task(stack)).unwrap();
    // spawner.spawn(weather_task(stack)).unwrap();
    // spawner.spawn(timetable_task(stack)).unwrap();
    // spawner.spawn(tick_task(stack)).unwrap();
}
