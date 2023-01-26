package com.oppzippy.openscq30.hilt

import com.oppzippy.openscq30.ui.deviceselection.models.BluetoothDeviceProvider
import dagger.Module
import dagger.Provides
import dagger.hilt.components.SingletonComponent
import dagger.hilt.testing.TestInstallIn
import io.mockk.mockk
import javax.inject.Singleton

@Module
@TestInstallIn(
    components = [SingletonComponent::class],
    replaces = [BluetoothDeviceProviderModule::class],
)
object MockBluetoothDeviceProviderModule {
    @Provides
    @Singleton
    fun provideBluetoothDeviceProvider(): BluetoothDeviceProvider {
        return mockk()
    }
}