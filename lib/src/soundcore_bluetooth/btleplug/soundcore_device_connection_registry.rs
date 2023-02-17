use std::collections::HashSet;
use std::sync::{Arc, Weak};
use std::vec;

use async_trait::async_trait;
use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::{stream, StreamExt};
use tokio::sync::Mutex;
use weak_table::weak_value_hash_map::Entry;
use weak_table::WeakValueHashMap;

use crate::soundcore_bluetooth::traits::{
    SoundcoreDeviceConnectionError, SoundcoreDeviceConnectionRegistry,
};

use super::soundcore_device_connection::BtlePlugSoundcoreDeviceConnection;
use super::BtlePlugSoundcoreDeviceConnectionDescriptor;

pub struct BtlePlugSoundcoreDeviceConnectionRegistry {
    manager: Manager,
    connections: Mutex<WeakValueHashMap<String, Weak<BtlePlugSoundcoreDeviceConnection>>>,
}

impl BtlePlugSoundcoreDeviceConnectionRegistry {
    pub fn new(manager: Manager) -> Self {
        Self {
            manager,
            connections: Mutex::new(WeakValueHashMap::new()),
        }
    }

    async fn all_connected(
        &self,
    ) -> Result<HashSet<BtlePlugSoundcoreDeviceConnectionDescriptor>, SoundcoreDeviceConnectionError>
    {
        let adapters = self.manager.adapters().await?;
        let peripherals = stream::iter(adapters)
            .filter_map(|adapter| async move { Self::adapter_to_peripherals(adapter).await })
            .flatten()
            .filter_map(
                |peripheral| async move { Self::filter_connected_peripherals(peripheral).await },
            )
            .filter_map(
                |peripheral| async move { Self::peripheral_to_descriptor(peripheral).await },
            )
            .collect::<HashSet<_>>()
            .await;
        Ok(peripherals)
    }

    async fn new_connection(
        &self,
        mac_address: &str,
    ) -> Result<Option<BtlePlugSoundcoreDeviceConnection>, SoundcoreDeviceConnectionError> {
        let adapters = self.manager.adapters().await?;
        let connections = stream::iter(adapters)
            .filter_map(|adapter| async move { Self::adapter_to_peripherals(adapter).await })
            .flatten()
            .filter_map(|peripheral| async move {
                if &peripheral.address().to_string() == mac_address {
                    Some(peripheral)
                } else {
                    None
                }
            })
            .filter_map(|peripheral| async move {
                Some(BtlePlugSoundcoreDeviceConnection::new(peripheral).await)
            })
            .collect::<Vec<_>>()
            .await;
        connections
            .into_iter()
            .next()
            .map(|connection_result| connection_result.map(Option::Some))
            .unwrap_or(Ok(None))
    }

    async fn adapter_to_peripherals(
        adapter: Adapter,
    ) -> Option<stream::Iter<vec::IntoIter<Peripheral>>> {
        match adapter.peripherals().await {
            Ok(peripherals) => Some(stream::iter(peripherals)),
            Err(err) => {
                tracing::warn!(
                    "failed to obtain peripherals for adapter {:?}: {err}",
                    adapter
                );
                None
            }
        }
    }

    async fn filter_connected_peripherals(peripheral: Peripheral) -> Option<Peripheral> {
        match peripheral.is_connected().await {
            Ok(is_connected) => {
                if is_connected {
                    Some(peripheral)
                } else {
                    None
                }
            }
            Err(err) => {
                tracing::warn!(
                    "unable to determine if peripheral {:?} is connected: {err}",
                    peripheral,
                );
                None
            }
        }
    }

    async fn peripheral_to_descriptor(
        peripheral: Peripheral,
    ) -> Option<BtlePlugSoundcoreDeviceConnectionDescriptor> {
        match peripheral.properties().await {
            Ok(Some(properties)) => Some(BtlePlugSoundcoreDeviceConnectionDescriptor::new(
                properties.local_name.unwrap_or_default(),
                properties.address.to_string(),
            )),
            Ok(None) => None,
            Err(err) => {
                tracing::warn!(
                    "failed to get peripheral {:?} properties: {err}",
                    peripheral,
                );
                None
            }
        }
    }
}

#[async_trait]
impl SoundcoreDeviceConnectionRegistry for BtlePlugSoundcoreDeviceConnectionRegistry {
    type DeviceConnectionType = BtlePlugSoundcoreDeviceConnection;
    type DescriptorType = BtlePlugSoundcoreDeviceConnectionDescriptor;

    async fn connection_descriptors(
        &self,
    ) -> Result<HashSet<Self::DescriptorType>, SoundcoreDeviceConnectionError> {
        self.all_connected().await
    }

    async fn connection(
        &self,
        mac_address: &str,
    ) -> Result<Option<Arc<Self::DeviceConnectionType>>, SoundcoreDeviceConnectionError> {
        match self.connections.lock().await.entry(mac_address.to_owned()) {
            Entry::Occupied(entry) => Ok(Some(entry.get().to_owned())),
            Entry::Vacant(entry) => {
                if let Some(connection) = self.new_connection(mac_address).await? {
                    let connection = Arc::new(connection);
                    entry.insert(connection.to_owned());
                    Ok(Some(connection))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
