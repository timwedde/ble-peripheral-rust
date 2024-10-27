use super::mac_utils;
use super::peripheral_delegate::PeripheralDelegate;
use super::{characteristic_utils::parse_characteristic, mac_extensions::uuid_to_cbuuid};
use crate::error::{Error, ErrorType};
use crate::gatt::peripheral_event::PeripheralEvent;
use crate::gatt::service::Service;
use objc2::msg_send_id;
use objc2::{rc::Retained, runtime::AnyObject, ClassType};
use objc2_core_bluetooth::{
    CBAdvertisementDataLocalNameKey, CBAdvertisementDataServiceUUIDsKey, CBCharacteristic,
    CBManager, CBManagerAuthorization, CBManagerState, CBMutableCharacteristic, CBMutableService,
    CBPeripheralManager,
};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSString};
use std::collections::HashMap;
use std::ffi::CString;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct PeripheralManager {
    cb_peripheral_manager: Retained<CBPeripheralManager>,
    peripheral_delegate: Retained<PeripheralDelegate>,
    cached_characteristics: HashMap<Uuid, Retained<CBMutableCharacteristic>>,
}

impl PeripheralManager {
    pub fn new(sender_tx: mpsc::Sender<PeripheralEvent>) -> Result<Self, Error> {
        if !is_authorized() {
            return Err(Error::from_type(ErrorType::PermissionDenied));
        }

        let delegate: Retained<PeripheralDelegate> = PeripheralDelegate::new(sender_tx);
        let label: CString = CString::new("CBqueue").unwrap();
        let queue: *mut std::ffi::c_void = unsafe {
            mac_utils::dispatch_queue_create(label.as_ptr(), mac_utils::DISPATCH_QUEUE_SERIAL)
        };
        let queue: *mut AnyObject = queue.cast();
        let peripheral_manager: Retained<CBPeripheralManager> = unsafe {
            msg_send_id![CBPeripheralManager::alloc(), initWithDelegate: &**delegate, queue: queue]
        };

        Ok(Self {
            cb_peripheral_manager: peripheral_manager,
            peripheral_delegate: delegate,
            cached_characteristics: HashMap::new(),
        })
    }

    pub fn is_powered(self: &Self) -> bool {
        unsafe {
            let state = self.cb_peripheral_manager.state();
            state == CBManagerState::PoweredOn
        }
    }

    pub async fn start_advertising(self: &Self, name: &str, uuids: &[Uuid]) -> Result<(), Error> {
        if self
            .peripheral_delegate
            .is_waiting_for_advertisement_result()
        {
            return Err(Error::from_string(
                "Already in progress".to_string(),
                ErrorType::CoreBluetooth,
            ));
        }

        let mut keys: Vec<&NSString> = vec![];
        let mut objects: Vec<Retained<AnyObject>> = vec![];

        unsafe {
            keys.push(CBAdvertisementDataLocalNameKey);
            objects.push(Retained::cast(NSString::from_str(name)));

            keys.push(CBAdvertisementDataServiceUUIDsKey);
            objects.push(Retained::cast(NSArray::from_vec(
                uuids.iter().map(|u| uuid_to_cbuuid(u.clone())).collect(),
            )));
        }

        let advertising_data: Retained<NSDictionary<NSString, AnyObject>> =
            NSDictionary::from_vec(&keys, objects);

        unsafe {
            self.cb_peripheral_manager
                .startAdvertising(Some(&advertising_data));
        }

        return self
            .peripheral_delegate
            .ensure_advertisement_started()
            .await;
    }

    pub fn stop_advertising(self: &Self) {
        unsafe {
            self.cb_peripheral_manager.stopAdvertising();
        }
    }

    pub fn is_advertising(self: &Self) -> bool {
        unsafe { self.cb_peripheral_manager.isAdvertising() }
    }

    pub async fn update_characteristic(
        &mut self,
        characteristic: Uuid,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        if let Some(char) = self.cached_characteristics.get(&characteristic) {
            unsafe {
                self.cb_peripheral_manager
                    .updateValue_forCharacteristic_onSubscribedCentrals(
                        &NSData::from_vec(value.clone()),
                        char,
                        None,
                    );
            }
        }
        return Ok(());
    }

    // Peripheral with cache value must only have Read permission, else it will crash
    // TODO: throw proper error, or catch Objc errors
    pub async fn add_service(&mut self, service: &Service) -> Result<(), Error> {
        if self
            .peripheral_delegate
            .is_waiting_for_service_result(service.uuid)
        {
            return Err(Error::from_string(
                "Already in progress".to_string(),
                ErrorType::CoreBluetooth,
            ));
        }

        unsafe {
            let mut characteristics: Vec<Retained<CBCharacteristic>> = Vec::new();

            for char in service.characteristics.iter() {
                let cb_char = parse_characteristic(char);
                characteristics.push(Retained::into_super(cb_char.clone()));
                self.cached_characteristics.insert(char.uuid, cb_char);
            }

            let mutable_service: Retained<CBMutableService> =
                CBMutableService::initWithType_primary(
                    CBMutableService::alloc(),
                    &uuid_to_cbuuid(service.uuid),
                    service.primary,
                );

            if !characteristics.is_empty() {
                let chars = NSArray::from_vec(characteristics);
                mutable_service.setCharacteristics(Some(&chars));
            }

            self.cb_peripheral_manager.addService(&mutable_service);

            return self
                .peripheral_delegate
                .ensure_service_added(service.uuid)
                .await;
        }
    }
}

pub fn is_authorized() -> bool {
    let authorization = unsafe { CBManager::authorization_class() };
    return authorization != CBManagerAuthorization::Restricted
        && authorization != CBManagerAuthorization::Denied;
}
