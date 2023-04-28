use openscq30_lib::packets::structures;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AmbientSoundMode {
    NoiseCanceling,
    Transparency,
    Normal,
}

impl From<structures::AmbientSoundMode> for AmbientSoundMode {
    fn from(value: structures::AmbientSoundMode) -> Self {
        match value {
            structures::AmbientSoundMode::NoiseCanceling => AmbientSoundMode::NoiseCanceling,
            structures::AmbientSoundMode::Transparency => AmbientSoundMode::Transparency,
            structures::AmbientSoundMode::Normal => AmbientSoundMode::Normal,
        }
    }
}

impl From<AmbientSoundMode> for structures::AmbientSoundMode {
    fn from(value: AmbientSoundMode) -> Self {
        match value {
            AmbientSoundMode::NoiseCanceling => structures::AmbientSoundMode::NoiseCanceling,
            AmbientSoundMode::Transparency => structures::AmbientSoundMode::Transparency,
            AmbientSoundMode::Normal => structures::AmbientSoundMode::Normal,
        }
    }
}
