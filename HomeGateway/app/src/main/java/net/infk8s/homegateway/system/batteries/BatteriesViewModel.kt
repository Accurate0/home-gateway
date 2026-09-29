package net.infk8s.homegateway.system.batteries

import android.app.Application
import java.time.Instant
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.BatteryHistoryQuery
import net.infk8s.homegateway.system.history.HistoryPoint
import net.infk8s.homegateway.system.history.HistorySeries
import net.infk8s.homegateway.system.history.HistoryViewModel
import net.infk8s.homegateway.system.history.historyPoints

class BatteriesViewModel(application: Application) : HistoryViewModel(application) {
    private val apollo = application.gateway.apollo

    override val tag = "BatteriesViewModel"

    override suspend fun fetch(since: Instant): List<HistorySeries> {
        val response = apollo.query(BatteryHistoryQuery(since.toString())).execute()
        val data = response.data
            ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load batteries")

        return data.entities
            .mapNotNull { it.toSeries() }
            .filter { it.points.isNotEmpty() }
            .sortedWith(compareBy({ it.current }, { it.name }))
    }

    private fun BatteryHistoryQuery.Entity.toSeries(): HistorySeries? = when {
        onLightEntity != null -> onLightEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onDoorEntity != null -> onDoorEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onPresenceEntity != null -> onPresenceEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onEnvironmentEntity != null -> onEnvironmentEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onEinkDisplayEntity != null -> onEinkDisplayEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onRobotVacuumEntity != null -> onRobotVacuumEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onGarageDoorEntity != null -> onGarageDoorEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        else -> null
    }

    private fun series(id: String, name: String, room: String?, points: List<HistoryPoint>) =
        HistorySeries(id, name, room, points.lastOrNull()?.value, points)
}
