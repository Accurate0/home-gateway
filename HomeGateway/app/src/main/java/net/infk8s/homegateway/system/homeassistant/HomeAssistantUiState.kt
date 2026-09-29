package net.infk8s.homegateway.system.homeassistant

sealed interface HomeAssistantUiState {
    data object Loading : HomeAssistantUiState
    data class Error(val message: String) : HomeAssistantUiState
    data class Loaded(val entities: List<HomeAssistantEntityUi>) : HomeAssistantUiState
}
