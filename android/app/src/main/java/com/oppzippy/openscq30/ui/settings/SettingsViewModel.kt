package com.oppzippy.openscq30.ui.settings

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.util.Log
import android.widget.Toast
import androidx.core.content.getSystemService
import androidx.lifecycle.ViewModel
import com.oppzippy.openscq30.features.autoconnect.AutoConnectService
import com.oppzippy.openscq30.features.preferences.Preferences
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

@HiltViewModel
class SettingsViewModel @Inject constructor(
    @ApplicationContext private val context: Context,
    private val preferences: Preferences,
) : ViewModel() {
    private val _autoConnect = MutableStateFlow(preferences.autoConnect)
    val autoConnect = _autoConnect.asStateFlow()
    private val _macAddressFilter = MutableStateFlow(preferences.macAddressFilter)
    val macAddressFilter = _macAddressFilter.asStateFlow()

    fun setAutoConnect(value: Boolean) {
        _autoConnect.value = value
        preferences.autoConnect = value
        val intent = Intent(context, AutoConnectService::class.java)
        if (value) {
            context.startService(intent)
        } else {
            context.stopService(intent)
        }
    }

    fun setMacAddressFilter(value: Boolean) {
        _macAddressFilter.value = value
        preferences.macAddressFilter = value
    }

    fun copyLogs() {
        val process = try {
            Log.i("SettingsViewModel", "exporting logs")
            Runtime.getRuntime().exec(arrayOf("logcat", "-d"))
        } catch (ex: Exception) {
            Toast.makeText(context, "Failed to execute logcat: ${ex.message}", Toast.LENGTH_SHORT).show()
            Log.e("SettingsViewModel", "Failed to execute logcat", ex)
            return
        }
        val logs = process.inputStream.bufferedReader().use {
            // we probably only need recent lines, and copying too much text to the clipboard makes it annoying to
            // deal with on a phone
            it.readLines().takeLast(200).joinToString("\n")
        }

        val clipboardManager = context.getSystemService<ClipboardManager>()
        if (clipboardManager != null) {
            clipboardManager.setPrimaryClip(ClipData.newPlainText("OpenSCQ30 logs", logs))
        } else {
            Toast.makeText(context, "Failed to get clipboard manager", Toast.LENGTH_SHORT).show()
        }
    }
}
