package net.infk8s.homegateway.transperth

import java.time.Instant

data class RouteUi(
    val id: String,
    val origin: String,
    val destination: String,
    val updatedAt: Instant?,
    val stale: Boolean,
    val departures: List<DepartureUi>,
)
