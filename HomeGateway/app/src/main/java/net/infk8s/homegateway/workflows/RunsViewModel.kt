package net.infk8s.homegateway.workflows

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import java.time.Instant
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.WorkflowRunTraceQuery
import net.infk8s.homegateway.graphql.WorkflowRunsQuery
import net.infk8s.homegateway.ui.parseInstant
import org.json.JSONArray
import org.json.JSONObject

data class RunUi(
    val id: String,
    val eventId: String,
    val slug: String,
    val name: String,
    val outcome: String,
    val dryRun: Boolean,
    val durationMs: Int,
    val error: String?,
    val startedAt: Instant?,
)

data class RunStepUi(
    val seq: Int,
    val depth: Int,
    val kind: String,
    val outcome: String,
    val guard: String?,
    val detail: String?,
    val error: String?,
    val durationUs: Int,
)

data class RunTraceUi(val steps: List<RunStepUi>, val trigger: String?)

sealed interface RunTraceState {
    data object Loading : RunTraceState
    data object Missing : RunTraceState
    data class Error(val message: String) : RunTraceState
    data class Loaded(val trace: RunTraceUi) : RunTraceState
}

sealed interface RunsUiState {
    data object Loading : RunsUiState
    data class Error(val message: String) : RunsUiState
    data class Loaded(val runs: List<RunUi>) : RunsUiState
}

class RunsViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val _state = MutableStateFlow<RunsUiState>(RunsUiState.Loading)
    val state: StateFlow<RunsUiState> = _state.asStateFlow()

    private val _traces = MutableStateFlow<Map<String, RunTraceState>>(emptyMap())
    val traces: StateFlow<Map<String, RunTraceState>> = _traces.asStateFlow()

    private var loading: Job? = null

    fun load(pendingEventId: String?) {
        loading?.cancel()
        loading = viewModelScope.launch {
            repeat(PENDING_POLL_LIMIT) { attempt ->
                val runs = fetch() ?: return@launch
                if (pendingEventId == null || runs.any { it.eventId == pendingEventId }) {
                    return@launch
                }

                if (attempt < PENDING_POLL_LIMIT - 1) {
                    delay(PENDING_POLL_MS)
                }
            }
        }
    }

    fun loadTrace(run: RunUi) {
        val existing = _traces.value[run.id]
        if (existing is RunTraceState.Loaded || existing is RunTraceState.Loading) {
            return
        }

        _traces.update { it + (run.id to RunTraceState.Loading) }

        viewModelScope.launch {
            val next = try {
                val response = apollo.query(WorkflowRunTraceQuery(run.eventId)).execute()
                val data = response.data
                    ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load trace")

                data.workflowRuns.firstOrNull { it.id == run.id }
                    ?.let { RunTraceState.Loaded(it.toUi()) }
                    ?: RunTraceState.Missing
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(TAG, "failed to load trace for ${run.id}", e)
                RunTraceState.Error(e.message ?: "Failed to load trace")
            }

            _traces.update { it + (run.id to next) }
        }
    }

    private suspend fun fetch(): List<RunUi>? = try {
        val response = apollo.query(WorkflowRunsQuery()).execute()
        val data = response.data
            ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load runs")

        data.workflowRuns.map { it.toUi() }.also { _state.value = RunsUiState.Loaded(it) }
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        Log.w(TAG, "failed to load runs", e)

        if (_state.value !is RunsUiState.Loaded) {
            _state.value = RunsUiState.Error(e.message ?: "Failed to load runs")
        }
        null
    }

    private fun WorkflowRunsQuery.WorkflowRun.toUi() = RunUi(
        id = id,
        eventId = eventId,
        slug = slug,
        name = name,
        outcome = outcome,
        dryRun = dryRun,
        durationMs = durationMs,
        error = error,
        startedAt = parseInstant(startedAt),
    )

    private fun WorkflowRunTraceQuery.WorkflowRun.toUi() = RunTraceUi(
        steps = steps.map {
            RunStepUi(
                seq = it.seq,
                depth = it.depth,
                kind = it.kind,
                outcome = it.outcome,
                guard = it.guard,
                detail = it.detail,
                error = it.error,
                durationUs = it.durationUs,
            )
        },
        trigger = trigger?.let(::prettyJson),
    )

    private fun prettyJson(value: Any): String = when (value) {
        is Map<*, *> -> JSONObject(value).toString(2)
        is List<*> -> JSONArray(value).toString(2)
        else -> value.toString()
    }

    private companion object {
        const val TAG = "RunsViewModel"
        const val PENDING_POLL_MS = 1_500L
        const val PENDING_POLL_LIMIT = 10
    }
}
