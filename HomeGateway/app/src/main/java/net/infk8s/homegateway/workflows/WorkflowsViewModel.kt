package net.infk8s.homegateway.workflows

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.DryRunWorkflowMutation
import net.infk8s.homegateway.graphql.SetWorkflowEnabledMutation
import net.infk8s.homegateway.graphql.WorkflowsQuery

data class WorkflowUi(
    val slug: String,
    val name: String,
    val group: String,
    val enabled: Boolean,
    val dryRun: Boolean,
    val reusable: Boolean,
    val modes: List<String>,
)

data class WorkflowGroupUi(val name: String, val workflows: List<WorkflowUi>)

sealed interface WorkflowsUiState {
    data object Loading : WorkflowsUiState
    data class Error(val message: String) : WorkflowsUiState
    data class Loaded(val groups: List<WorkflowGroupUi>, val enabled: Int, val total: Int) : WorkflowsUiState
}

class WorkflowsViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val workflows = MutableStateFlow<List<WorkflowUi>?>(null)

    private val _state = MutableStateFlow<WorkflowsUiState>(WorkflowsUiState.Loading)
    val state: StateFlow<WorkflowsUiState> = _state.asStateFlow()

    private val _dryRunError = MutableStateFlow<String?>(null)
    val dryRunError: StateFlow<String?> = _dryRunError.asStateFlow()

    private val _dryRunning = MutableStateFlow(false)
    val dryRunning: StateFlow<Boolean> = _dryRunning.asStateFlow()

    fun refresh() {
        viewModelScope.launch {
            try {
                val response = apollo.query(WorkflowsQuery()).execute()
                val data = response.data
                    ?: throw IllegalStateException(
                        response.errors?.firstOrNull()?.message ?: "Failed to load workflows",
                    )

                publish(data.workflows.map { it.toUi() })
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(TAG, "failed to load workflows", e)

                if (workflows.value == null) {
                    _state.value = WorkflowsUiState.Error(e.message ?: "Failed to load workflows")
                }
            }
        }
    }

    fun setEnabled(slug: String, enabled: Boolean) {
        setLocal(slug, enabled)

        viewModelScope.launch {
            val succeeded = try {
                val response = apollo.mutation(SetWorkflowEnabledMutation(slug, enabled)).execute()
                if (response.hasErrors()) {
                    Log.w(TAG, "setWorkflowEnabled($slug) returned errors: ${response.errors}")
                }

                response.data != null && !response.hasErrors()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(TAG, "setWorkflowEnabled($slug) failed", e)
                false
            }

            if (!succeeded) {
                setLocal(slug, !enabled)
            }
        }
    }

    fun dryRun(workflow: WorkflowUi, onStarted: (String) -> Unit) {
        _dryRunError.value = null
        _dryRunning.value = true

        viewModelScope.launch {
            try {
                val response = apollo.mutation(DryRunWorkflowMutation(workflow.slug)).execute()
                val eventId = response.data?.runWorkflow?.toString()
                    ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "dry run failed")

                onStarted(eventId)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(TAG, "dry run of ${workflow.slug} failed", e)
                _dryRunError.value = "${workflow.name}: ${e.message}"
            } finally {
                _dryRunning.value = false
            }
        }
    }

    private fun setLocal(slug: String, enabled: Boolean) {
        val current = workflows.value ?: return

        publish(current.map { if (it.slug == slug) it.copy(enabled = enabled) else it })
    }

    private fun publish(updated: List<WorkflowUi>) {
        workflows.value = updated

        val groups = updated
            .groupBy { it.group }
            .toSortedMap()
            .map { (name, list) -> WorkflowGroupUi(name, list) }

        _state.value = WorkflowsUiState.Loaded(groups, updated.count { it.enabled }, updated.size)
    }

    private fun WorkflowsQuery.Workflow.toUi() = WorkflowUi(
        slug = slug,
        name = name,
        group = group,
        enabled = enabled,
        dryRun = dryRun,
        reusable = reusable,
        modes = modes.map { it.rawValue.lowercase() },
    )

    private companion object {
        const val TAG = "WorkflowsViewModel"
    }
}
