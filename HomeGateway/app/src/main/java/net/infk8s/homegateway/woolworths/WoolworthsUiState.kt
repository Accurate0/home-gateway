package net.infk8s.homegateway.woolworths

sealed interface WoolworthsUiState {
    data object Loading : WoolworthsUiState
    data class Error(val message: String) : WoolworthsUiState
    data class Loaded(val products: List<WoolworthsProductUi>) : WoolworthsUiState
}
