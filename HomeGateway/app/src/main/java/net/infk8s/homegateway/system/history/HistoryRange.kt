package net.infk8s.homegateway.system.history

import java.time.Instant
import java.time.temporal.ChronoUnit

enum class HistoryRange(val label: String, val days: Long) {
    DAY("24h", 1),
    WEEK("7d", 7),
    FORTNIGHT("2w", 14),
    MONTH("30d", 30),
    QUARTER("90d", 90);

    fun since(): Instant = Instant.now().truncatedTo(ChronoUnit.HOURS).minus(days, ChronoUnit.DAYS)
}
