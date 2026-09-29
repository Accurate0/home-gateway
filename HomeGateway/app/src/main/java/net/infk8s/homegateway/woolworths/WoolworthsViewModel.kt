package net.infk8s.homegateway.woolworths

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
import net.infk8s.homegateway.graphql.WoolworthsPriceHistoryQuery
import net.infk8s.homegateway.graphql.WoolworthsProductsQuery
import net.infk8s.homegateway.graphql.WoolworthsUpdatesSubscription
import net.infk8s.homegateway.graphql.keepLive
import net.infk8s.homegateway.system.history.HistoryPoint
import net.infk8s.homegateway.system.history.HistoryRange
import net.infk8s.homegateway.system.history.historyPoints

class WoolworthsViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val _state = MutableStateFlow<WoolworthsUiState>(WoolworthsUiState.Loading)
    val state: StateFlow<WoolworthsUiState> = _state.asStateFlow()

    private val _histories = MutableStateFlow<Map<Int, List<HistoryPoint>>>(emptyMap())
    val histories: StateFlow<Map<Int, List<HistoryPoint>>> = _histories.asStateFlow()

    val range = HistoryRange.QUARTER

    suspend fun run() {
        keepLive(TAG, ::load, ::listen) { e ->
            if (_state.value !is WoolworthsUiState.Loaded) {
                _state.value = WoolworthsUiState.Error(e.message ?: "Failed to load products")
            }
        }
    }

    fun loadHistory(productId: Int) {
        viewModelScope.launch { fetchHistory(productId) }
    }

    private suspend fun load() {
        val response = apollo.query(WoolworthsProductsQuery()).execute()
        val data = response.data
            ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load products")

        _state.value = WoolworthsUiState.Loaded(
            data.woolworths.products.map { WoolworthsProductUi(it.productId, it.name, it.price) },
        )
    }

    private suspend fun listen() {
        apollo.subscription(WoolworthsUpdatesSubscription()).toFlow()
            .collect { response ->
                val update = response.data?.events?.onWoolworthsUpdate ?: return@collect

                load()
                if (update.productId in _histories.value) {
                    fetchHistory(update.productId)
                }
            }
    }

    private suspend fun fetchHistory(productId: Int) {
        try {
            val response = apollo.query(WoolworthsPriceHistoryQuery(productId, range.since().toString())).execute()
            val data = response.data
                ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load price history")

            val points = historyPoints(data.woolworths.priceHistory, { it.time }, { it.price })
            _histories.value += productId to points
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Log.w(TAG, "failed to load price history for $productId", e)
        }
    }

    private companion object {
        const val TAG = "WoolworthsViewModel"
    }
}
