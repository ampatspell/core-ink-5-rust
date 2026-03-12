use defmt::info;
use embedded_storage::{ReadStorage, nor_flash::NorFlash};
use esp_storage::FlashStorage;
use littlefs_rust::{Config, Error, Filesystem, OpenFlags, Storage};

#[derive(Debug)]
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

pub type EspStorageFilesystem = Filesystem<EspStorageFlash>;

static BLOCK_SIZE: u32 = 4096;

pub fn create_filesystem() -> EspStorageFilesystem {
    let config = || Config::new(BLOCK_SIZE, 1024); // 4MB
    let storage = || EspStorageFlash::new(BLOCK_SIZE);

    // butcher magic "littlefs" to get Error::Corrupt
    // let data: [u8; BLOCK_SIZE as usize] = [0; BLOCK_SIZE as usize];
    // storage().write(0, 0, &data).unwrap();

    let fs = Filesystem::mount(storage(), config());
    let fs = match fs {
        Ok(fs) => {
            info!("Filesystem mounted");
            fs
        }
        Err(_) => {
            info!("Failed to mount filesystem");
            let mut storage = storage();
            let config = config();
            Filesystem::format(&mut storage, &config).unwrap();
            info!("Formatted");
            let fs = Filesystem::mount(storage, config).unwrap();
            info!("Mounted after format");
            fs
        }
    };

    fs
}

pub fn read_write_filesystem(fs: &EspStorageFilesystem) {
    let file = fs.open("/hello", OpenFlags::READ);
    match file {
        Ok(file) => {
            info!("File exists");
            let mut buf: [u8; 8] = [0; 8];
            file.read(&mut buf).unwrap();
            info!("Bytes {}", buf);
        }
        Err(_) => {
            info!("File open error");
            fs.write_file("/hello", b"Hello").unwrap();
            info!("Wrote /hello");
        }
    }
}
