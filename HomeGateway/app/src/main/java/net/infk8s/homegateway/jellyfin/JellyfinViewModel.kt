package net.infk8s.homegateway.jellyfin

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.JellyfinSessionsQuery
import net.infk8s.homegateway.graphql.JellyfinUpdatesSubscription
import net.infk8s.homegateway.graphql.keepLive

class JellyfinViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val sessions = LinkedHashMap<String, JellyfinSessionUi>()

    private val _state = MutableStateFlow<JellyfinUiState>(JellyfinUiState.Loading)
    val state: StateFlow<JellyfinUiState> = _state.asStateFlow()

    suspend fun run() {
        keepLive(TAG, ::load, ::listen) { e ->
            if (_state.value !is JellyfinUiState.Loaded) {
                _state.value = JellyfinUiState.Error(e.message ?: "Failed to load Jellyfin sessions")
            }
        }
    }

    private suspend fun load() {
        val response = apollo.query(JellyfinSessionsQuery()).execute()
        val data = response.data
            ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load Jellyfin sessions")

        sessions.clear()
        data.jellyfin.nowPlaying.forEach { sessions[it.sessionId] = it.toUi() }
        publish()
    }

    private suspend fun listen() {
        apollo.subscription(JellyfinUpdatesSubscription()).toFlow()
            .collect { response ->
                val update = response.data?.events?.onJellyfinUpdate ?: return@collect

                if (update.state == STOPPED) {
                    sessions.remove(update.sessionId)
                } else {
                    sessions[update.sessionId] = update.toUi()
                }

                publish()
            }
    }

    private fun publish() {
        _state.value = JellyfinUiState.Loaded(sessions.values.toList())
    }

    private fun JellyfinSessionsQuery.NowPlaying.toUi() = JellyfinSessionUi(
        sessionId = sessionId,
        user = user,
        device = device,
        client = client,
        itemName = itemName,
        itemType = itemType,
        seriesName = seriesName,
        season = season,
        episode = episode,
        positionSeconds = positionSeconds,
        runtimeSeconds = runtimeSeconds,
        playMethod = playMethod,
        paused = paused,
    )

    private fun JellyfinUpdatesSubscription.OnJellyfinUpdate.toUi() = JellyfinSessionUi(
        sessionId = sessionId,
        user = user,
        device = device,
        client = client,
        itemName = itemName,
        itemType = itemType,
        seriesName = seriesName,
        season = season,
        episode = episode,
        positionSeconds = positionSeconds,
        runtimeSeconds = runtimeSeconds,
        playMethod = playMethod,
        paused = state == PAUSED,
    )

    private companion object {
        const val TAG = "JellyfinViewModel"
        const val STOPPED = "stopped"
        const val PAUSED = "paused"
    }
}
