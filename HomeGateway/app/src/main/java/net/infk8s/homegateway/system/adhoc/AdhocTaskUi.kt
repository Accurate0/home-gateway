package net.infk8s.homegateway.system.adhoc

import java.time.Instant

data class AdhocTaskUi(
    val id: String,
    val ordinal: Int,
    val name: String,
    val flag: String?,
    val completedAt: Instant?,
    val durationMs: Int?,
    val pending: Boolean,
    val checksumDrifted: Boolean,
)
