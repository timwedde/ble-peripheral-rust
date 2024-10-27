mod characteristic_utils;
mod error;
mod peripheral_manager;
mod win_event_handler;
mod win_utils;

use self::peripheral_manager::PeripheralManager;
use crate::error::Error;
use crate::gatt::peripheral_event::PeripheralEvent;
use crate::gatt::service::Service;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

pub struct Peripheral {
    peripheral_manager: PeripheralManager,
}

impl Peripheral {
    pub async fn new(sender_tx: Sender<PeripheralEvent>) -> Result<Self, Error> {
        Ok(Self {
            peripheral_manager: PeripheralManager::new(sender_tx).await,
        })
    }

    pub async fn is_powered(&self) -> Result<bool, Error> {
        Ok(self.peripheral_manager.is_powered().await?)
    }

    pub async fn start_advertising(&self, name: &str, uuids: &[Uuid]) -> Result<(), Error> {
        if let Err(err) = self.peripheral_manager.start_advertising(name, uuids).await {
            return Err(Error::from(err));
        }
        Ok(())
    }

    pub async fn stop_advertising(&self) -> Result<(), Error> {
        if let Err(err) = self.peripheral_manager.stop_advertising().await {
            return Err(Error::from(err));
        }
        Ok(())
    }

    pub async fn is_advertising(&self) -> Result<bool, Error> {
        Ok(self.peripheral_manager.is_advertising().await?)
    }

    pub async fn add_service(&mut self, service: &Service) -> Result<(), Error> {
        if let Err(err) = self.peripheral_manager.add_service(service).await {
            return Err(Error::from(err));
        }
        Ok(())
    }

    pub async fn update_characteristic(
        &mut self,
        characteristic: Uuid,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        if let Err(err) = self
            .peripheral_manager
            .update_characteristic(characteristic, value)
            .await
        {
            return Err(Error::from(err));
        }
        Ok(())
    }
}
