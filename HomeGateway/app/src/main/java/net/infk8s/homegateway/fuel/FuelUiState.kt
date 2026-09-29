package net.infk8s.homegateway.fuel

sealed interface FuelUiState {
    data object Loading : FuelUiState
    data class Error(val message: String) : FuelUiState
    data class Loaded(val sites: List<FuelSiteUi>) : FuelUiState
}
