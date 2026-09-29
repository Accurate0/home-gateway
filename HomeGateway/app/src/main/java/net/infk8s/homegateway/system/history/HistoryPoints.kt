package net.infk8s.homegateway.system.history

import net.infk8s.homegateway.ui.parseInstant

fun <T> historyPoints(history: List<T>, time: (T) -> Any, value: (T) -> Double?): List<HistoryPoint> =
    history
        .mapNotNull { point ->
            val at = parseInstant(time(point)) ?: return@mapNotNull null
            val reading = value(point) ?: return@mapNotNull null

            HistoryPoint(at, reading)
        }
        .sortedBy { it.time }
