package net.infk8s.homegateway.system.plants

import android.app.Application
import java.time.Instant
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.PlantHistoryQuery
import net.infk8s.homegateway.system.history.HistorySeries
import net.infk8s.homegateway.system.history.HistoryViewModel
import net.infk8s.homegateway.system.history.historyPoints

class PlantsViewModel(application: Application) : HistoryViewModel(application) {
    private val apollo = application.gateway.apollo

    override val tag = "PlantsViewModel"

    override suspend fun fetch(since: Instant): List<HistorySeries> {
        val response = apollo.query(PlantHistoryQuery(since.toString())).execute()
        val data = response.data
            ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load plants")

        return data.entities
            .mapNotNull { it.onPlantEntity }
            .map { plant ->
                val points = historyPoints(plant.history, { it.time }, { it.soilMoisture })

                HistorySeries(plant.id, plant.name, plant.room, plant.soilMoisture ?: points.lastOrNull()?.value, points)
            }
            .sortedBy { it.name }
    }
}
