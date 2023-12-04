use std::{mem, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::FutureExt;
use macaddr::MacAddr6;
use tokio::sync::{mpsc, watch, Mutex};
use tracing::{trace, warn};
use uuid::Uuid;

use crate::{
    api::connection::{Connection, ConnectionStatus},
    devices::standard::{
        packets::outbound::{OutboundPacketBytes, RequestFirmwareVersionPacket},
        state,
        structures::{CustomButtonModel, EqualizerConfiguration, HearId, SoundModes},
    },
    futures::{Futures, JoinHandle},
};
use crate::{
    api::{self},
    devices::standard::packets::{inbound::InboundPacket, outbound::RequestStatePacket},
    devices::standard::state::DeviceState,
};

use super::{
    soundcore_command::CommandResponse,
    soundcore_dispatcher::{
        DefaultDispatcher, SetCustomButtonModel, SetEqualizerConfiguration, SetHearId,
        SetSoundModes,
    },
};

pub struct SoundcoreDevice<ConnectionType, FuturesType>
where
    ConnectionType: Connection,
    FuturesType: Futures,
{
    connection: Arc<ConnectionType>,
    state_sender: Arc<Mutex<watch::Sender<DeviceState>>>,
    join_handle: FuturesType::JoinHandleType,
    set_sound_modes: Box<dyn SetSoundModes>,
    set_equalizer_configuration: Box<dyn SetEqualizerConfiguration>,
    set_hear_id: Box<dyn SetHearId>,
    set_custom_button_model: Box<dyn SetCustomButtonModel>,
}

impl<ConnectionType, FuturesType> SoundcoreDevice<ConnectionType, FuturesType>
where
    ConnectionType: Connection,
    FuturesType: Futures,
{
    pub async fn new(connection: Arc<ConnectionType>) -> crate::Result<Self> {
        let mut inbound_receiver = connection.inbound_packets_channel().await?;
        let initial_state = Self::fetch_initial_state(&connection, &mut inbound_receiver).await?;

        // TODO consider making this a part of fetch_initial_state
        // For devices that don't include the firmware version in their state update packet, we need to request it
        if initial_state.firmware_version.is_none() {
            connection
                .write_with_response(&RequestFirmwareVersionPacket::new().bytes())
                .await?;
        }

        struct Dispatchers {
            set_sound_mode: Box<dyn SetSoundModes>,
            set_equalizer_configuration: Box<dyn SetEqualizerConfiguration>,
            set_hear_id: Box<dyn SetHearId>,
            set_custom_button_model: Box<dyn SetCustomButtonModel>,
        }

        let dispatchers = match initial_state.device_profile.custom_dispatchers.map(|f| f()) {
            Some(custom_dispatchers) => Dispatchers {
                set_sound_mode: custom_dispatchers
                    .set_sound_mode
                    .unwrap_or_else(|| Box::new(DefaultDispatcher)),
                set_equalizer_configuration: custom_dispatchers
                    .set_equalizer_configuration
                    .unwrap_or_else(|| Box::new(DefaultDispatcher)),
                set_hear_id: custom_dispatchers
                    .set_hear_id
                    .unwrap_or_else(|| Box::new(DefaultDispatcher)),
                set_custom_button_model: custom_dispatchers
                    .set_custom_button_model
                    .unwrap_or_else(|| Box::new(DefaultDispatcher)),
            },
            None => Dispatchers {
                set_sound_mode: Box::new(DefaultDispatcher),
                set_equalizer_configuration: Box::new(DefaultDispatcher),
                set_hear_id: Box::new(DefaultDispatcher),
                set_custom_button_model: Box::new(DefaultDispatcher),
            },
        };

        let (state_sender, _) = watch::channel(initial_state);
        let state_sender = Arc::new(Mutex::new(state_sender));

        let join_handle =
            Self::spawn_inbound_packet_handler(inbound_receiver, state_sender.to_owned());

        Ok(Self {
            connection,
            join_handle,
            state_sender,
            set_sound_modes: dispatchers.set_sound_mode,
            set_equalizer_configuration: dispatchers.set_equalizer_configuration,
            set_hear_id: dispatchers.set_hear_id,
            set_custom_button_model: dispatchers.set_custom_button_model,
        })
    }

    fn spawn_inbound_packet_handler(
        mut inbound_receiver: mpsc::Receiver<Vec<u8>>,
        state_sender_lock: Arc<Mutex<watch::Sender<DeviceState>>>,
    ) -> FuturesType::JoinHandleType {
        FuturesType::spawn(async move {
            while let Some(packet_bytes) = inbound_receiver.recv().await {
                match InboundPacket::new(&packet_bytes) {
                    Ok(packet) => {
                        let state_sender = state_sender_lock.lock().await;
                        let state = state_sender.borrow();
                        let new_state = state::transform_state(packet, &state);
                        if new_state != *state {
                            trace!(event = "state_update", old_state = ?state, new_state = ?new_state);
                            mem::drop(state);
                            state_sender.send_replace(new_state);
                        }
                    }
                    Err(err) => warn!("failed to parse packet: {err:?}"),
                }
            }
        })
    }

    async fn fetch_initial_state(
        connection: &ConnectionType,
        inbound_receiver: &mut mpsc::Receiver<Vec<u8>>,
    ) -> crate::Result<DeviceState> {
        for i in 0..3 {
            connection
                .write_without_response(&RequestStatePacket::new().bytes())
                .await?;

            let state_future = async {
                while let Some(packet_bytes) = inbound_receiver.recv().await {
                    match InboundPacket::new(&packet_bytes) {
                        Ok(InboundPacket::StateUpdate(packet)) => {
                            return Some(packet.into());
                        }
                        Err(err) => warn!("failed to parse packet: {err:?}"),
                        _ => (), // Known packet, but not the one we're looking for
                    };
                }
                None
            };

            futures::select! {
                state = state_future.fuse() => if let Some(state) = state { return Ok(state) },
                _ = FuturesType::sleep(Duration::from_secs(1)).fuse() =>
                    warn!("fetch_initial_state: didn't receive response after 1 second on try #{i}"),
            }
        }
        Err(crate::Error::NoResponse)
    }

    async fn handle_response(
        &self,
        response: CommandResponse,
        state_sender: &watch::Sender<DeviceState>,
    ) -> crate::Result<()> {
        self.send_packets(response.packets).await?;
        state_sender.send_replace(response.new_state);
        Ok(())
    }

    async fn send_packets(&self, packets: impl IntoIterator<Item = Vec<u8>>) -> crate::Result<()> {
        for packet in packets {
            self.connection.write_with_response(&packet).await?;
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl<ConnectionType, FuturesType> api::device::Device
    for SoundcoreDevice<ConnectionType, FuturesType>
where
    ConnectionType: Connection,
    FuturesType: Futures,
{
    async fn subscribe_to_state_updates(&self) -> watch::Receiver<DeviceState> {
        self.state_sender.lock().await.subscribe()
    }

    async fn mac_address(&self) -> crate::Result<MacAddr6> {
        self.connection.mac_address().await
    }

    async fn name(&self) -> crate::Result<String> {
        self.connection.name().await
    }

    fn connection_status(&self) -> watch::Receiver<ConnectionStatus> {
        self.connection.connection_status()
    }

    fn service_uuid(&self) -> Uuid {
        self.connection.service_uuid()
    }

    async fn state(&self) -> DeviceState {
        self.state_sender.lock().await.borrow().to_owned()
    }

    async fn set_sound_modes(&self, sound_modes: SoundModes) -> crate::Result<()> {
        let state_sender = self.state_sender.lock().await;
        let state = state_sender.borrow().to_owned();
        if state.device_profile.sound_mode.is_none() {
            return Err(crate::Error::FeatureNotSupported {
                feature_name: "sound modes",
            });
        }
        let Some(prev_sound_modes) = state.sound_modes else {
            return Err(crate::Error::MissingData {
                name: "sound modes",
            });
        };
        if prev_sound_modes == sound_modes {
            return Ok(());
        }

        let response = self.set_sound_modes.set_sound_modes(state, sound_modes)?;
        self.handle_response(response, &state_sender).await?;
        Ok(())
    }

    async fn set_equalizer_configuration(
        &self,
        equalizer_configuration: EqualizerConfiguration,
    ) -> crate::Result<()> {
        let state_sender = self.state_sender.lock().await;
        let state = state_sender.borrow().to_owned();

        if state.device_profile.num_equalizer_channels == 0 {
            return Err(crate::Error::FeatureNotSupported {
                feature_name: "equalizer",
            });
        }
        if equalizer_configuration
            .volume_adjustments()
            .adjustments()
            .len()
            != state.device_profile.num_equalizer_bands
        {
            return Err(crate::Error::FeatureNotSupported {
                feature_name: "wrong number of equalizer bands",
            });
        }
        if equalizer_configuration == state.equalizer_configuration {
            return Ok(());
        }

        let response = self
            .set_equalizer_configuration
            .set_equalizer_configuration(state, equalizer_configuration)?;
        self.handle_response(response, &state_sender).await?;
        Ok(())
    }

    async fn set_hear_id(&self, hear_id: HearId) -> crate::Result<()> {
        let state_sender = self.state_sender.lock().await;
        let state = state_sender.borrow().to_owned();

        if !state.device_profile.has_hear_id {
            return Err(crate::Error::FeatureNotSupported {
                feature_name: "hear id",
            });
        }

        let response = self.set_hear_id.set_hear_id(state, hear_id)?;
        self.handle_response(response, &state_sender).await?;
        Ok(())
    }

    async fn set_custom_button_model(
        &self,
        custom_button_model: CustomButtonModel,
    ) -> crate::Result<()> {
        let state_sender = self.state_sender.lock().await;
        let state = state_sender.borrow().to_owned();

        if !state.device_profile.has_custom_button_model {
            return Err(crate::Error::FeatureNotSupported {
                feature_name: "custom button model",
            });
        }

        let prev_custom_button_model =
            state.custom_button_model.ok_or(crate::Error::MissingData {
                name: "custom button model",
            })?;
        if custom_button_model == prev_custom_button_model {
            return Ok(());
        }

        let response = self
            .set_custom_button_model
            .set_custom_button_model(state, custom_button_model)?;
        self.handle_response(response, &state_sender).await?;
        Ok(())
    }
}

impl<ConnectionType, FuturesType> Drop for SoundcoreDevice<ConnectionType, FuturesType>
where
    ConnectionType: Connection,
    FuturesType: Futures,
{
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

impl<ConnectionType, FuturesType> std::fmt::Debug for SoundcoreDevice<ConnectionType, FuturesType>
where
    ConnectionType: Connection,
    FuturesType: Futures,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundcoreDevice").finish()
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use macaddr::MacAddr6;
    use tokio::sync::mpsc;

    use super::SoundcoreDevice;
    use crate::{
        api::device::Device,
        devices::standard::structures::{
            AmbientSoundMode, CustomNoiseCanceling, EqualizerConfiguration, NoiseCancelingMode,
            SoundModes, VolumeAdjustments,
        },
        futures::TokioFutures,
        stub::connection::StubConnection,
    };

    fn example_state_update_packet() -> Vec<u8> {
        vec![
            0x09, 0xff, 0x00, 0x00, 0x01, 0x01, 0x01, 0x46, 0x00, 0x05, 0x00, 0xfe, 0xfe, 0x3c,
            0xb4, 0x8f, 0xa0, 0x8e, 0xb4, 0x74, 0x88, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x30, 0x32, 0x2e, 0x33, 0x30, 0x33, 0x30, 0x32,
            0x39, 0x30, 0x38, 0x36, 0x45, 0x43, 0x38, 0x32, 0x46, 0x31, 0x32, 0x41, 0x43, 0x30,
        ]
    }

    async fn create_test_connection() -> (Arc<StubConnection>, mpsc::Sender<Vec<u8>>) {
        let connection = Arc::new(StubConnection::new());
        connection
            .set_name_return(Ok("Soundcore Q30".to_string()))
            .await;
        connection.set_mac_address_return(Ok(MacAddr6::nil())).await;

        let (sender, receiver) = mpsc::channel(100);
        connection.set_inbound_packets_channel(Ok(receiver)).await;
        (connection, sender)
    }

    #[tokio::test]
    async fn test_new_with_example_state_update_packet() {
        let (connection, sender) = create_test_connection().await;
        connection.push_write_return(Ok(())).await;
        tokio::spawn(async move {
            sender.send(example_state_update_packet()).await.unwrap();
        });
        let device = SoundcoreDevice::<_, TokioFutures>::new(connection)
            .await
            .unwrap();
        let state = device.state().await;
        let sound_modes = state.sound_modes.unwrap();
        assert_eq!(AmbientSoundMode::Normal, sound_modes.ambient_sound_mode);
        assert_eq!(
            NoiseCancelingMode::Transport,
            sound_modes.noise_canceling_mode
        );
        assert!(state.equalizer_configuration.preset_profile().is_none());
        assert_eq!(
            &VolumeAdjustments::new([-6.0, 6.0, 2.3, 4.0, 2.2, 6.0, -0.4, 1.6]).unwrap(),
            state.equalizer_configuration.volume_adjustments(),
        )
    }

    #[tokio::test]
    async fn test_new_with_retry() {
        let (connection, sender) = create_test_connection().await;
        connection.push_write_return(Ok(())).await;
        connection.push_write_return(Ok(())).await;
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(1500)).await;
            sender.send(example_state_update_packet()).await.unwrap();
        });
        let connection_clone = connection.clone();
        SoundcoreDevice::<_, TokioFutures>::new(connection_clone)
            .await
            .unwrap();
        assert_eq!(0, connection.write_return_queue_length().await);
    }

    #[tokio::test]
    async fn test_new_max_retries() {
        let (connection, _) = create_test_connection().await;
        // for the purposes of this test, we don't care how many times it retries. we only care that it stops at some point.
        for _ in 0..100 {
            connection.push_write_return(Ok(())).await;
        }

        let connection_clone = connection.clone();
        let result = SoundcoreDevice::<_, TokioFutures>::new(connection_clone).await;
        assert_eq!(true, result.is_err());
    }

    #[tokio::test]
    async fn test_ambient_sound_mode_update_packet() {
        let (connection, sender) = create_test_connection().await;
        connection.push_write_return(Ok(())).await;
        let sender_copy = sender.clone();
        tokio::spawn(async move {
            sender_copy
                .send(example_state_update_packet())
                .await
                .unwrap();
        });
        let device = SoundcoreDevice::<_, TokioFutures>::new(connection)
            .await
            .unwrap();
        let state = device.state().await;
        let sound_modes = state.sound_modes.unwrap();
        assert_eq!(AmbientSoundMode::Normal, sound_modes.ambient_sound_mode);
        assert_eq!(
            NoiseCancelingMode::Transport,
            sound_modes.noise_canceling_mode
        );

        tokio::spawn(async move {
            sender
                .send(vec![
                    0x09, 0xff, 0x00, 0x00, 0x01, 0x06, 0x01, 0x0e, 0x00, 0x00, 0x01, 0x01, 0x00,
                    0x20,
                ])
                .await
                .unwrap();
        });
        // wait for the packet to be handled asynchronously
        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = device.state().await;
        let sound_modes = state.sound_modes.unwrap();

        assert_eq!(
            AmbientSoundMode::NoiseCanceling,
            sound_modes.ambient_sound_mode,
        );
        assert_eq!(
            NoiseCancelingMode::Outdoor,
            sound_modes.noise_canceling_mode,
        );
    }

    #[tokio::test]
    async fn test_set_sound_mode_called_twice() {
        let (connection, sender) = create_test_connection().await;
        // request state update packet
        connection.push_write_return(Ok(())).await;
        // first set_sound_modes. second call should not send a packet.
        connection.push_write_return(Ok(())).await;
        sender.send(example_state_update_packet()).await.unwrap();

        let device = SoundcoreDevice::<_, TokioFutures>::new(connection.to_owned())
            .await
            .unwrap();
        let sound_modes = SoundModes {
            custom_noise_canceling: CustomNoiseCanceling::new(10),
            ..Default::default()
        };
        device.set_sound_modes(sound_modes).await.unwrap();
        device.set_sound_modes(sound_modes).await.unwrap();
    }

    #[tokio::test]
    async fn test_set_equalizer_configuration_called_twice() {
        let (connection, sender) = create_test_connection().await;
        // request state update packet
        connection.push_write_return(Ok(())).await;
        // first set_equalizer_configuration. second call should not send a packet.
        connection.push_write_return(Ok(())).await;
        sender.send(example_state_update_packet()).await.unwrap();

        let device = SoundcoreDevice::<_, TokioFutures>::new(connection.to_owned())
            .await
            .unwrap();
        let equalizer_configuration = EqualizerConfiguration::new_custom_profile(
            VolumeAdjustments::new([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]).unwrap(),
        );
        device
            .set_equalizer_configuration(equalizer_configuration.to_owned())
            .await
            .unwrap();
        device
            .set_equalizer_configuration(equalizer_configuration)
            .await
            .unwrap();
    }
}
