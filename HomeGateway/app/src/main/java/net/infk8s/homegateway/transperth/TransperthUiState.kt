package net.infk8s.homegateway.transperth

sealed interface TransperthUiState {
    data object Loading : TransperthUiState
    data class Error(val message: String) : TransperthUiState
    data class Loaded(val routes: List<RouteUi>) : TransperthUiState
}
