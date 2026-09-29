package net.infk8s.homegateway.transperth

import java.time.Instant

data class DepartureUi(
    val line: String,
    val headsign: String,
    val platform: String?,
    val departsAt: Instant,
    val delayMinutes: Int?,
    val live: Boolean,
)
