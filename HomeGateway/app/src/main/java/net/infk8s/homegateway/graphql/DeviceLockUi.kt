package net.infk8s.homegateway.graphql

import java.time.Duration
import java.time.Instant
import net.infk8s.homegateway.graphql.type.DeviceLockSource

data class DeviceLockUi(val source: DeviceLockSource, val expiresAt: Instant?) {
    fun activeAt(now: Instant): Boolean = expiresAt == null || expiresAt.isAfter(now)

    fun sourceLabel(): String = source.rawValue.lowercase().replaceFirstChar { it.uppercase() }

    fun remaining(now: Instant): String {
        val expires = expiresAt ?: return "until unlocked"
        val minutes = Duration.between(now, expires).plusSeconds(59).toMinutes().coerceAtLeast(0)
        val hours = minutes / 60

        return if (hours == 0L) "${minutes}m left" else "${hours}h ${minutes % 60}m left"
    }
}
