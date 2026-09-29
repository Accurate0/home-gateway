package net.infk8s.homegateway.system.history

data class HistorySeries(
    val id: String,
    val name: String,
    val room: String?,
    val current: Double?,
    val points: List<HistoryPoint>,
)
