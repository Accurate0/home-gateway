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

        return data.homeEntities
            .mapNotNull { it.toSeries() }
            .filter { it.points.isNotEmpty() }
            .sortedWith(compareBy({ it.current }, { it.name }))
    }

    private fun BatteryHistoryQuery.HomeEntity.toSeries(): HistorySeries? = when {
        onHomeLightEntity != null -> onHomeLightEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onHomeDoorEntity != null -> onHomeDoorEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onHomePresenceEntity != null -> onHomePresenceEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onHomeEnvironmentEntity != null -> onHomeEnvironmentEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onHomeEinkDisplayEntity != null -> onHomeEinkDisplayEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onHomeRobotVacuumEntity != null -> onHomeRobotVacuumEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        onHomeGarageDoorEntity != null -> onHomeGarageDoorEntity.run {
            series(id, name, room, historyPoints(battery?.history.orEmpty(), { it.time }, { it.batteryPercentage }))
        }

        else -> null
    }

    private fun series(id: String, name: String, room: String?, points: List<HistoryPoint>) =
        HistorySeries(id, name, room, points.lastOrNull()?.value, points)
}
