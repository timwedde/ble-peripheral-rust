mod characteristic_utils;
mod mac_extensions;
mod mac_utils;
pub mod peripheral_delegate;
mod peripheral_manager;

use crate::{
    error::Error,
    gatt::{peripheral_event::PeripheralEvent, service::Service},
};
use peripheral_manager::PeripheralManager;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

pub struct Peripheral {
    peripheral_manager: PeripheralManager,
}

impl Peripheral {
    pub async fn new(sender_tx: Sender<PeripheralEvent>) -> Result<Self, Error> {
        let peripheral_manager = PeripheralManager::new(sender_tx).unwrap();
        Ok(Peripheral { peripheral_manager })
    }

    pub async fn is_powered(&mut self) -> Result<bool, Error> {
        return Ok(self.peripheral_manager.is_powered());
    }

    pub async fn is_advertising(&mut self) -> Result<bool, Error> {
        return Ok(self.peripheral_manager.is_advertising());
    }

    pub async fn start_advertising(&mut self, name: &str, uuids: &[Uuid]) -> Result<(), Error> {
        return self.peripheral_manager.start_advertising(name, uuids).await;
    }

    pub async fn stop_advertising(&mut self) -> Result<(), Error> {
        return Ok(self.peripheral_manager.stop_advertising());
    }

    pub async fn add_service(&mut self, service: &Service) -> Result<(), Error> {
        return self.peripheral_manager.add_service(service).await;
    }

    pub async fn update_characteristic(
        &mut self,
        characteristic: Uuid,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        return self
            .peripheral_manager
            .update_characteristic(characteristic, value)
            .await;
    }
}
