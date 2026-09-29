package net.infk8s.homegateway.jellyfin

sealed interface JellyfinUiState {
    data object Loading : JellyfinUiState
    data class Error(val message: String) : JellyfinUiState
    data class Loaded(val sessions: List<JellyfinSessionUi>) : JellyfinUiState
}
