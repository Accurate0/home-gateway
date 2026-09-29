package net.infk8s.homegateway.system.homeassistant

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import java.time.Instant
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.HomeAssistantEntitiesQuery
import net.infk8s.homegateway.graphql.HomeAssistantUpdatesSubscription
import net.infk8s.homegateway.graphql.keepLive
import net.infk8s.homegateway.ui.parseInstant

class HomeAssistantViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val latest = LinkedHashMap<String, HomeAssistantEntityUi>()

    private val _state = MutableStateFlow<HomeAssistantUiState>(HomeAssistantUiState.Loading)
    val state: StateFlow<HomeAssistantUiState> = _state.asStateFlow()

    suspend fun run() {
        keepLive(TAG, ::loadSnapshot, ::subscribe) { e ->
            if (_state.value !is HomeAssistantUiState.Loaded) {
                _state.value = HomeAssistantUiState.Error(e.message ?: "Failed to load Home Assistant entities")
            }
        }
    }

    private suspend fun loadSnapshot() {
        val response = apollo.query(HomeAssistantEntitiesQuery()).execute()
        val data = response.data
            ?: throw IllegalStateException(
                response.errors?.firstOrNull()?.message ?: "Failed to load Home Assistant entities",
            )

        data.homeAssistant.entities.forEach { entity ->
            val time = parseInstant(entity.time) ?: return@forEach
            upsert(HomeAssistantEntityUi(entity.entityId, entity.state, time))
        }

        publish()
    }

    private suspend fun subscribe() {
        apollo.subscription(HomeAssistantUpdatesSubscription()).toFlow()
            .collect { response ->
                val update = response.data?.events?.onHomeAssistantUpdate ?: return@collect

                upsert(HomeAssistantEntityUi(update.entityId, update.state, Instant.now()))
                publish()
            }
    }

    private fun upsert(entity: HomeAssistantEntityUi) {
        val existing = latest[entity.entityId]
        if (existing != null && existing.time >= entity.time) {
            return
        }

        latest[entity.entityId] = entity
    }

    private fun publish() {
        _state.value = HomeAssistantUiState.Loaded(latest.values.sortedByDescending { it.time })
    }

    private companion object {
        const val TAG = "HomeAssistantViewModel"
    }
}
