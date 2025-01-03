use anyhow::{anyhow, Context};
use openscq30_lib::api::device::{Device, DeviceRegistry};

use crate::{
    actions,
    settings::{Config, SettingsFile},
};

use super::State;

pub async fn delete_quick_preset<T>(
    state: &State<T>,
    settings_file: &SettingsFile<Config>,
    quick_preset_name: &str,
) -> anyhow::Result<()>
where
    T: DeviceRegistry + 'static,
{
    let Some(device) = state.selected_device() else {
        anyhow::bail!("cannot delete quick preset while not connected to a device");
    };
    let device_model = device
        .state()
        .await
        .serial_number
        .map(|sn| sn.model_number().to_owned())
        .ok_or(anyhow!("missing device serial number"))?;

    settings_file
        .edit(|settings| {
            settings.remove_quick_preset(device_model.to_owned(), quick_preset_name);
        })
        .context("edit settings")?;
    actions::refresh_quick_presets(state, settings_file, &device_model)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use openscq30_lib::devices::standard::{state::DeviceState, structures::SerialNumber};
    use uuid::Uuid;

    use crate::{
        actions::{State, StateUpdate},
        mock::{MockDevice, MockDeviceRegistry},
        objects::GlibNamedQuickPresetValue,
        settings::{Config, QuickPreset, SettingsFile},
    };

    use super::delete_quick_preset;

    #[gtk::test]
    async fn it_works() {
        crate::load_resources();
        let registry = MockDeviceRegistry::new();
        let (state, mut receiver) = State::new(registry);
        let mut device = MockDevice::new();
        device.expect_service_uuid().return_const(Uuid::default());
        device.expect_state().return_const(DeviceState {
            serial_number: Some(SerialNumber("0123".into())),
            ..Default::default()
        });
        *state.selected_device.borrow_mut() = Some(Rc::new(device));

        let dir = tempfile::tempdir().unwrap();
        let settings_file = SettingsFile::new(dir.path().join("config.toml"));
        settings_file
            .edit(|config: &mut Config| {
                config.set_quick_preset("0123".into(), "test".into(), QuickPreset::default());
            })
            .unwrap();
        delete_quick_preset(&state, &settings_file, "test")
            .await
            .unwrap();

        let state_update = receiver.recv().await.unwrap();
        if let StateUpdate::SetQuickPresets(quick_presets) = state_update {
            assert_eq!(Vec::<GlibNamedQuickPresetValue>::new(), quick_presets);
        } else {
            panic!("StateUpdate was not RefreshQuickPresets: {state_update:?}");
        }
    }
}
