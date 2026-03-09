use crate::ble::BlePins;
use core::cell::RefCell;
use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_time::{Duration, Timer};
use esp_radio::ble::controller::BleConnector;
use heapless::Deque;
use trouble_host::prelude::*;
use trouble_host::{Address, Host, HostResources};

pub fn spawn_ble_tasks<'a>(spawner: &Spawner, pins: BlePins) {
    let connector = BleConnector::new(pins.bt, Default::default()).unwrap();
    let controller = ExternalController::new(connector);
    spawner.spawn(scanner_task(controller)).unwrap();
}

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

#[embassy_executor::task]
async fn scanner_task(controller: ExternalController<BleConnector<'static>, 20>) {
    let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
    info!("Ble own address {:?}", address);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();

    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);

    let Host {
        central,
        mut runner,
        ..
    } = stack.build();

    let printer = Printer {
        seen: RefCell::new(Deque::new()),
    };

    let mut scanner = Scanner::new(central);
    let _ = join(runner.run_with_handler(&printer), async {
        let mut config = ScanConfig::default();
        config.active = true;
        config.phys = PhySet::M1;
        config.interval = Duration::from_secs(1);
        config.window = Duration::from_secs(1);
        let mut _session = scanner.scan(&config).await.unwrap();
        loop {
            Timer::after(Duration::from_secs(1)).await;
        }
    })
    .await;
}

struct Printer {
    seen: RefCell<Deque<BdAddr, 128>>,
}

impl EventHandler for Printer {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        let mut seen = self.seen.borrow_mut();
        while let Some(Ok(report)) = it.next() {
            if seen.iter().find(|b| b.raw() == report.addr.raw()).is_none() {
                info!("ble discovered: {:?}", report.addr);
                if seen.is_full() {
                    seen.pop_front();
                }
                seen.push_back(report.addr).unwrap();
            }
        }
    }
}
