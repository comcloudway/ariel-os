use core::ops::Range;
use embassy_embedded_hal::adapter::BlockingAsync;
use esp_bootloader_esp_idf::partitions;
use esp_hal::peripherals::OptionalPeripherals;
use esp_storage::{FlashStorage, FlashStorageError};

pub type Flash = BlockingAsync<FlashStorage<'static>>;
pub type FlashError = FlashStorageError;

/// obtains a valid flash storage instance and the flash range associated with the data/nvs partition
pub fn init(p: &mut OptionalPeripherals) -> (Flash, Range<u32>) {
    let mut flash = esp_storage::FlashStorage::new(
        p.FLASH.take().expect("Storage crate requires flash access"),
    );

    let mut pt_mem = [0u8; partitions::PARTITION_TABLE_MAX_LEN];

    let pt = partitions::read_partition_table(&mut flash, &mut pt_mem).unwrap();

    let nvs = pt
        .find_partition(partitions::PartitionType::Data(
            partitions::DataPartitionSubType::Nvs,
        ))
        .unwrap()
        .unwrap();

    let nvs_begin = nvs.offset();
    let nvs_end = nvs_begin + nvs.len();

    (BlockingAsync::new(flash), nvs_begin..nvs_end)
}
