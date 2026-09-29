package net.infk8s.homegateway.system.adhoc

import java.time.Instant

data class AdhocCronTaskUi(
    val id: String,
    val name: String,
    val schedule: String,
    val flag: String?,
    val nextRunAt: Instant?,
    val lastRunAt: Instant?,
    val durationMs: Int?,
    val rowsAffected: Int?,
    val outcome: String?,
)
