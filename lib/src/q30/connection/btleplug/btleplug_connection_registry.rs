use std::collections::HashSet;
use std::rc::{Rc, Weak};
use std::time::Duration;
use std::vec;

use async_trait::async_trait;
use btleplug::api::{BDAddr, Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::{stream, StreamExt};
use macaddr::MacAddr6;
use tokio::sync::Mutex;
use weak_table::weak_value_hash_map::Entry;
use weak_table::WeakValueHashMap;

use crate::api::connection::{ConnectionRegistry, GenericConnectionDescriptor};

use super::btleplug_connection::BtlePlugConnection;
use super::mac_address::{IntoBDAddr, IntoMacAddr};

pub struct BtlePlugConnectionRegistry {
    manager: Manager,
    connections: Mutex<WeakValueHashMap<MacAddr6, Weak<BtlePlugConnection>>>,
}

impl BtlePlugConnectionRegistry {
    pub fn new(manager: Manager) -> Self {
        Self {
            manager,
            connections: Mutex::new(WeakValueHashMap::new()),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn all_connected(&self) -> crate::Result<HashSet<GenericConnectionDescriptor>> {
        let adapters = self.manager.adapters().await?;
        for adapter in adapters.iter() {
            tracing::debug!("starting scan");
            adapter
                .start_scan(ScanFilter {
                    services: vec![crate::device_utils::SERVICE_UUID],
                })
                .await?;
            // The Soundcore Q30 seems to advertise around every .3s
            // Wait some extra time to be safe
            tokio::time::sleep(Duration::from_secs(1)).await;
            adapter.stop_scan().await?;
            tracing::debug!("scan finished");
        }
        let peripherals = stream::iter(adapters)
            .filter_map(|adapter| async move { Self::adapter_to_peripherals(adapter).await })
            .flatten()
            .map(|adapter_and_peripheral| adapter_and_peripheral.1)
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
        mac_address: BDAddr,
    ) -> crate::Result<Option<BtlePlugConnection>> {
        let adapters = self.manager.adapters().await?;
        let connections = stream::iter(adapters)
            .filter_map(|adapter| async move { Self::adapter_to_peripherals(adapter).await })
            .flatten()
            .filter_map(|adapter_and_peripheral| async move {
                if adapter_and_peripheral.1.address() == mac_address {
                    Some(adapter_and_peripheral)
                } else {
                    None
                }
            })
            .filter_map(|adapter_and_peripheral| async move {
                Some(
                    BtlePlugConnection::new(adapter_and_peripheral.0, adapter_and_peripheral.1)
                        .await,
                )
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
    ) -> Option<impl stream::Stream<Item = (Adapter, Peripheral)>> {
        match adapter.peripherals().await {
            Ok(peripherals) => {
                Some(stream::iter(peripherals).map(move |x| (adapter.to_owned(), x)))
            }
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
    ) -> Option<GenericConnectionDescriptor> {
        match peripheral.properties().await {
            Ok(Some(properties)) => Some(GenericConnectionDescriptor::new(
                properties.local_name.unwrap_or_default(),
                properties.address.into_mac_addr(),
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

#[async_trait(?Send)]
impl ConnectionRegistry for BtlePlugConnectionRegistry {
    type ConnectionType = BtlePlugConnection;
    type DescriptorType = GenericConnectionDescriptor;

    async fn connection_descriptors(&self) -> crate::Result<HashSet<Self::DescriptorType>> {
        self.all_connected().await
    }

    async fn connection(
        &self,
        mac_address: MacAddr6,
    ) -> crate::Result<Option<Rc<Self::ConnectionType>>> {
        match self.connections.lock().await.entry(mac_address.to_owned()) {
            Entry::Occupied(entry) => Ok(Some(entry.get().to_owned())),
            Entry::Vacant(entry) => {
                if let Some(connection) = self.new_connection(mac_address.into_bd_addr()).await? {
                    let connection = Rc::new(connection);
                    entry.insert(connection.to_owned());
                    Ok(Some(connection))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
