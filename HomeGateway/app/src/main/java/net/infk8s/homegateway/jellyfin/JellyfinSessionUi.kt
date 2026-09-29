package net.infk8s.homegateway.jellyfin

data class JellyfinSessionUi(
    val sessionId: String,
    val user: String,
    val device: String,
    val client: String,
    val itemName: String,
    val itemType: String,
    val seriesName: String?,
    val season: Int?,
    val episode: Int?,
    val positionSeconds: Double?,
    val runtimeSeconds: Double?,
    val playMethod: String?,
    val paused: Boolean,
)
