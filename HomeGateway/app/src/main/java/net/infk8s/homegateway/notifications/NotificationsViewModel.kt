package net.infk8s.homegateway.notifications

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.infk8s.homegateway.graphql.ApolloProvider
import net.infk8s.homegateway.graphql.PushNotificationsQuery
import net.infk8s.homegateway.graphql.type.NotificationInteractionKind
import net.infk8s.homegateway.graphql.type.PushNotificationActionKind

data class NotificationUi(
    val id: String,
    val title: String,
    val body: String,
    val category: String,
    val acknowledgeable: Boolean,
    val sendCount: Int,
    val nextReminderAt: String?,
    val acknowledgedAt: String?,
    val createdAt: String,
    val interactions: List<NotificationInteractionKind>,
)

sealed interface NotificationsUiState {
    data object Loading : NotificationsUiState
    data class Error(val message: String) : NotificationsUiState
    data class Loaded(val notifications: List<NotificationUi>) : NotificationsUiState
}

class NotificationsViewModel : ViewModel() {
    private val apollo = ApolloProvider.client

    private val _state = MutableStateFlow<NotificationsUiState>(NotificationsUiState.Loading)
    val state: StateFlow<NotificationsUiState> = _state.asStateFlow()

    fun refresh() {
        viewModelScope.launch { load() }
    }

    fun acknowledge(id: String) {
        viewModelScope.launch {
            NotificationInteractions.record(id, NotificationInteractionKind.ACKNOWLEDGED)
            load()
        }
    }

    private suspend fun load() {
        try {
            val response = apollo.query(PushNotificationsQuery()).execute()
            val data = response.data
                ?: throw IllegalStateException(
                    response.errors?.firstOrNull()?.message ?: "Failed to load notifications",
                )

            _state.value = NotificationsUiState.Loaded(data.pushNotifications.map { it.toUi() })
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Log.w(TAG, "failed to load notifications", e)

            if (_state.value !is NotificationsUiState.Loaded) {
                _state.value = NotificationsUiState.Error(e.message ?: "Failed to load notifications")
            }
        }
    }

    private fun PushNotificationsQuery.PushNotification.toUi() = NotificationUi(
        id = id.toString(),
        title = title,
        body = body,
        category = category,
        acknowledgeable = actions.any { it.kind == PushNotificationActionKind.ACKNOWLEDGE },
        sendCount = sendCount,
        nextReminderAt = nextReminderAt?.toString(),
        acknowledgedAt = acknowledgedAt?.toString(),
        createdAt = createdAt.toString(),
        interactions = interactions.map { it.kind },
    )

    private companion object {
        const val TAG = "NotificationsViewModel"
    }
}
