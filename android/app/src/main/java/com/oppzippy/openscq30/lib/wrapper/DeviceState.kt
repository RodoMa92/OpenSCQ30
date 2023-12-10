package com.oppzippy.openscq30.lib.wrapper

import com.oppzippy.openscq30.lib.protobuf.customButtonModelOrNull
import com.oppzippy.openscq30.lib.protobuf.deviceState
import com.oppzippy.openscq30.lib.protobuf.firmwareVersionOrNull
import com.oppzippy.openscq30.lib.protobuf.hearIdOrNull
import com.oppzippy.openscq30.lib.protobuf.soundModesOrNull

data class DeviceState(
    val deviceProfile: DeviceProfile,
    val battery: Battery,
    val equalizerConfiguration: EqualizerConfiguration,
    val soundModes: SoundModes?,
    val ageRange: UByte?,
    val gender: UByte?,
    val hearId: HearId?,
    val customButtonModel: CustomButtonModel?,
    val firmwareVersion: FirmwareVersion?,
    val serialNumber: String?,
) {
    companion object // used for static extension methods in tests

    fun supportsDynamicRangeCompression(): Boolean {
        if (deviceProfile.hasDynamicRangeCompression) {
            if (firmwareVersion == null) {
                return false
            }
            val minAllowedFirmwareVersion =
                deviceProfile.dynamicRangeCompressionMinFirmwareVersion ?: return true
            return firmwareVersion >= minAllowedFirmwareVersion
        }
        return false
    }

    fun toProtobuf(): com.oppzippy.openscq30.lib.protobuf.DeviceState {
        return deviceState {
            deviceProfile = this@DeviceState.deviceProfile.toProtobuf()
            battery = this@DeviceState.battery.toProtobuf()
            equalizerConfiguration = this@DeviceState.equalizerConfiguration.toProtobuf()
            this@DeviceState.soundModes?.let { soundModes = it.toProtobuf() }
            this@DeviceState.ageRange?.let { ageRange = it.toInt() }
            this@DeviceState.gender?.let { gender = it.toInt() }
            this@DeviceState.hearId?.let { hearId = it.toProtobuf() }
            this@DeviceState.customButtonModel?.let { customButtonModel = it.toProtobuf() }
            this@DeviceState.firmwareVersion?.let { firmwareVersion = it.toProtobuf() }
            this@DeviceState.serialNumber?.let { serialNumber = it }
        }
    }
}

fun com.oppzippy.openscq30.lib.protobuf.DeviceState.toKotlin(): DeviceState {
    return DeviceState(
        deviceProfile = deviceProfile.toKotlin(),
        battery = battery.toKotlin(),
        equalizerConfiguration = equalizerConfiguration.toKotlin(),
        soundModes = soundModesOrNull?.toKotlin(),
        ageRange = if (hasAgeRange()) ageRange.toUByte() else null,
        gender = if (hasGender()) gender.toUByte() else null,
        hearId = hearIdOrNull?.toKotlin(),
        customButtonModel = customButtonModelOrNull?.toKotlin(),
        firmwareVersion = firmwareVersionOrNull?.toKotlin(),
        serialNumber = if (hasSerialNumber()) serialNumber else null,
    )
}
