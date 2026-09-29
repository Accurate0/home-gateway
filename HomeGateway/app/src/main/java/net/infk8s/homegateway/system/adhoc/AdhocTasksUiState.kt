package net.infk8s.homegateway.system.adhoc

sealed interface AdhocTasksUiState {
    data object Loading : AdhocTasksUiState
    data class Error(val message: String) : AdhocTasksUiState
    data class Loaded(val cron: List<AdhocCronTaskUi>, val tasks: List<AdhocTaskUi>) : AdhocTasksUiState
}
