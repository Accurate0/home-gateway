package net.infk8s.homegateway.system.history

sealed interface HistoryUiState {
    data object Loading : HistoryUiState
    data class Error(val message: String) : HistoryUiState
    data class Loaded(val series: List<HistorySeries>) : HistoryUiState
}
