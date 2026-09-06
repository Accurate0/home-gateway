package net.infk8s.homegateway.notifications

import android.app.NotificationManager

enum class NotificationCategory(
    val id: String,
    val displayName: String,
    val importance: Int,
) {
    ALARM("home_gateway_alarm", "Alarms", NotificationManager.IMPORTANCE_HIGH),
    DOOR("home_gateway_door", "Doors", NotificationManager.IMPORTANCE_HIGH),
    WATCHDOG("home_gateway_watchdog", "Sensor health", NotificationManager.IMPORTANCE_DEFAULT),
    GENERAL("home_gateway_general", "Home Gateway", NotificationManager.IMPORTANCE_DEFAULT),
    ;

    val groupKey: String
        get() = "net.infk8s.homegateway.$id"

    companion object {
        fun from(value: String?): NotificationCategory =
            entries.firstOrNull { it.name.equals(value, ignoreCase = true) } ?: GENERAL
    }
}
