package net.infk8s.homegateway.system.history

import java.time.Instant

data class HistoryPoint(val time: Instant, val value: Double)
