package net.infk8s.homegateway.fuel

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.FuelSitesQuery
import net.infk8s.homegateway.graphql.FuelWatchUpdatesSubscription
import net.infk8s.homegateway.graphql.keepLive

class FuelViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val _state = MutableStateFlow<FuelUiState>(FuelUiState.Loading)
    val state: StateFlow<FuelUiState> = _state.asStateFlow()

    suspend fun run() {
        keepLive(TAG, ::load, ::listen) { e ->
            if (_state.value !is FuelUiState.Loaded) {
                _state.value = FuelUiState.Error(e.message ?: "Failed to load fuel prices")
            }
        }
    }

    private suspend fun load() {
        val response = apollo.query(FuelSitesQuery()).execute()
        val data = response.data
            ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load fuel prices")

        _state.value = FuelUiState.Loaded(data.fuelwatch.sites.map { it.toUi() })
    }

    private suspend fun listen() {
        apollo.subscription(FuelWatchUpdatesSubscription()).toFlow()
            .collect { response ->
                if (response.data?.events?.onFuelWatchUpdate != null) {
                    load()
                }
            }
    }

    private fun FuelSitesQuery.Site.toUi() = FuelSiteUi(
        siteId = siteId,
        name = name,
        brand = brand,
        suburb = suburb,
        address = address,
        price = price,
        priceTomorrow = priceTomorrow,
        latitude = latitude,
        longitude = longitude,
    )

    private companion object {
        const val TAG = "FuelViewModel"
    }
}
