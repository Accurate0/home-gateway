package net.infk8s.homegateway.graphql

import android.util.Log
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay

suspend fun keepLive(
    tag: String,
    load: suspend () -> Unit,
    listen: suspend () -> Unit,
    onFailure: (Exception) -> Unit,
) {
    var backoffMs = INITIAL_BACKOFF_MS

    while (true) {
        try {
            load()
            backoffMs = INITIAL_BACKOFF_MS
            listen()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Log.w(tag, "live connection lost, reconnecting in ${backoffMs}ms", e)
            onFailure(e)

            delay(backoffMs)
            backoffMs = (backoffMs * 2).coerceAtMost(MAX_BACKOFF_MS)
        }
    }
}

private const val INITIAL_BACKOFF_MS = 1_000L
private const val MAX_BACKOFF_MS = 30_000L
