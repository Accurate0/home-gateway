package net.infk8s.homegateway.system.history

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import java.time.Instant
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

abstract class HistoryViewModel(application: Application) : AndroidViewModel(application) {
    private val _state = MutableStateFlow<HistoryUiState>(HistoryUiState.Loading)
    val state: StateFlow<HistoryUiState> = _state.asStateFlow()

    private val _range = MutableStateFlow(HistoryRange.FORTNIGHT)
    val range: StateFlow<HistoryRange> = _range.asStateFlow()

    private var loading: Job? = null

    protected abstract val tag: String

    protected abstract suspend fun fetch(since: Instant): List<HistorySeries>

    fun select(range: HistoryRange) {
        if (_range.value == range) {
            return
        }

        _range.value = range
        _state.value = HistoryUiState.Loading
        refresh()
    }

    fun refresh() {
        loading?.cancel()

        loading = viewModelScope.launch {
            try {
                _state.value = HistoryUiState.Loaded(fetch(_range.value.since()))
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(tag, "failed to load history", e)

                if (_state.value !is HistoryUiState.Loaded) {
                    _state.value = HistoryUiState.Error(e.message ?: "Failed to load history")
                }
            }
        }
    }
}
