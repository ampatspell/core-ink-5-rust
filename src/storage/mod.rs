use defmt::info;
use embassy_executor::Spawner;
use embedded_storage::{ReadStorage, nor_flash::NorFlash};
use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;
use heapless::String;
use littlefs_rust::{Config, Error, Filesystem, OpenFlags, Storage};
use no_std_strings::{str32, str64};

pub struct EspStorageFlash {
    storage: FlashStorage,
    block_size: u32,
}

impl EspStorageFlash {
    pub fn new(block_size: u32) -> Self {
        let storage = FlashStorage::new();
        Self {
            storage,
            block_size,
        }
    }
}

impl Storage for EspStorageFlash {
    fn read(&mut self, block: u32, offset: u32, buf: &mut [u8]) -> Result<(), Error> {
        let offset = (self.block_size * block) + offset;
        let read = self.storage.read(offset, buf);
        match read {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Io),
        }
    }

    fn write(&mut self, block: u32, offset: u32, data: &[u8]) -> Result<(), Error> {
        let offset = (self.block_size * block) + offset;
        let write = self.storage.write(offset, data);
        match write {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Io),
        }
    }

    fn erase(&mut self, block: u32) -> Result<(), Error> {
        let size = self.block_size;
        let from = size * block;
        let to = from + size;
        let erase = self.storage.erase(from, to);
        match erase {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Io),
        }
    }
}

pub fn spawn_storage_task(_spawner: &Spawner, pins: StoragePins) {
    // spawner.spawn(flash_task(pins)).unwrap();

    let config = Config::new(4096, 1024); // 4MB
    let mut storage = EspStorageFlash::new(config.block_size);
    Filesystem::format(&mut storage, &config).unwrap();
    let fs = Filesystem::mount(storage, config);
    match fs {
        Ok(fs) => {
            info!("Has fs");
            let opened = fs
                .open("/hello", OpenFlags::WRITE | OpenFlags::READ)
                .unwrap();
            opened.write("Hello".as_bytes()).unwrap();

            let mut buf: [u8; 32] = [0; 32];
            opened.read(&mut buf).unwrap();
            info!("read {:?}", buf);
        }
        Err((err, _)) => {
            info!("Failed to mount");
        }
    };
}

pub struct StoragePins {
    pub flash: FLASH<'static>,
}
