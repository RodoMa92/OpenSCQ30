package com.oppzippy.openscq30.features.soundcoredevice.impl

import android.bluetooth.BluetoothGatt
import com.oppzippy.openscq30.features.soundcoredevice.api.SoundcoreDevice
import com.oppzippy.openscq30.lib.wrapper.SoundcoreDeviceState
import kotlinx.coroutines.CoroutineScope
import javax.inject.Inject

class SoundcoreDeviceFactoryImpl @Inject constructor() : SoundcoreDeviceFactory {
    override fun createSoundcoreDevice(
        gatt: BluetoothGatt,
        callbackHandler: SoundcoreDeviceCallbackHandler,
        scope: CoroutineScope,
        deviceState: SoundcoreDeviceState,
    ): SoundcoreDevice {
        return SoundcoreDeviceImpl(gatt, callbackHandler, scope, deviceState)
    }
}
