package net.infk8s.homegateway.dashboard

import android.content.Context
import androidx.core.content.edit

class DashboardPreferences(context: Context) {
    private val preferences = context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    var mode: DashboardMode
        get() = preferences.getString(KEY_MODE, null)
            ?.let { stored -> DashboardMode.entries.firstOrNull { it.name == stored } }
            ?: DashboardMode.TYPE
        set(value) = preferences.edit { putString(KEY_MODE, value.name) }

    private companion object {
        const val PREFERENCES = "dashboard"
        const val KEY_MODE = "mode"
    }
}
