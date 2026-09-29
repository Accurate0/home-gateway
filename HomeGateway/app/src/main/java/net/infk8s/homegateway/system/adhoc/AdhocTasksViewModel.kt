package net.infk8s.homegateway.system.adhoc

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.apollographql.apollo.api.Mutation
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.AdhocTasksQuery
import net.infk8s.homegateway.graphql.RunAdhocCronTaskMutation
import net.infk8s.homegateway.graphql.RunPendingAdhocTasksMutation
import net.infk8s.homegateway.ui.parseInstant

class AdhocTasksViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val _state = MutableStateFlow<AdhocTasksUiState>(AdhocTasksUiState.Loading)
    val state: StateFlow<AdhocTasksUiState> = _state.asStateFlow()

    private val _running = MutableStateFlow<String?>(null)
    val running: StateFlow<String?> = _running.asStateFlow()

    private val _runError = MutableStateFlow<String?>(null)
    val runError: StateFlow<String?> = _runError.asStateFlow()

    fun refresh() {
        viewModelScope.launch { load() }
    }

    fun runCron(name: String) {
        run(name, RunAdhocCronTaskMutation(name))
    }

    fun runPending() {
        run(PENDING_KEY, RunPendingAdhocTasksMutation())
    }

    private fun run(key: String, mutation: Mutation<*>) {
        if (_running.value != null) {
            return
        }

        _running.value = key
        _runError.value = null

        viewModelScope.launch {
            try {
                val response = apollo.mutation(mutation).execute()
                if (response.hasErrors()) {
                    Log.w(TAG, "${mutation.name()} returned errors: ${response.errors}")
                    _runError.value = response.errors?.firstOrNull()?.message
                }

                delay(REFRESH_DELAY_MS)
                load()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(TAG, "${mutation.name()} failed", e)
                _runError.value = e.message ?: "Run failed"
            } finally {
                _running.value = null
            }
        }
    }

    private suspend fun load() {
        try {
            val response = apollo.query(AdhocTasksQuery()).execute()
            val data = response.data
                ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load adhoc tasks")

            _state.value = AdhocTasksUiState.Loaded(
                cron = data.adhocCronTasks.map { it.toUi() },
                tasks = data.adhocTasks.map { it.toUi() }.sortedBy { it.ordinal },
            )
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Log.w(TAG, "failed to load adhoc tasks", e)

            if (_state.value !is AdhocTasksUiState.Loaded) {
                _state.value = AdhocTasksUiState.Error(e.message ?: "Failed to load adhoc tasks")
            }
        }
    }

    private fun AdhocTasksQuery.AdhocCronTask.toUi() = AdhocCronTaskUi(
        id = id,
        name = name,
        schedule = schedule,
        flag = flag,
        nextRunAt = parseInstant(nextRunAt),
        lastRunAt = parseInstant(lastRunAt),
        durationMs = durationMs,
        rowsAffected = rowsAffected,
        outcome = outcome,
    )

    private fun AdhocTasksQuery.AdhocTask.toUi() = AdhocTaskUi(
        id = id,
        ordinal = ordinal,
        name = name,
        flag = flag,
        completedAt = parseInstant(completedAt),
        durationMs = durationMs,
        pending = pending,
        checksumDrifted = checksumDrifted,
    )

    companion object {
        const val PENDING_KEY = "pending"
        private const val TAG = "AdhocTasksViewModel"
        private const val REFRESH_DELAY_MS = 2_000L
    }
}
