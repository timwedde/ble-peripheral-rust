use bluer::gatt::local::CharacteristicControl;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct CharNotifyHandler {
    pub service_uuid: Uuid,
    pub characteristic_uuid: Uuid,
    pub control: CharacteristicControl,
}
