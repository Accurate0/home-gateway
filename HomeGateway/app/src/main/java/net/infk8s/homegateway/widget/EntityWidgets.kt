package net.infk8s.homegateway.widget

import android.content.Context
import android.util.Log
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.glance.GlanceId
import androidx.glance.appwidget.GlanceAppWidgetManager
import androidx.glance.appwidget.state.updateAppWidgetState
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.CancellationException
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.EntitySnapshot
import net.infk8s.homegateway.graphql.EntityUi

object EntityWidgets {
    val ENTITY_KEY = stringPreferencesKey("entity_key")
    val ERROR = stringPreferencesKey("error")

    private const val TAG = "EntityWidgets"
    private const val REFRESH_WORK = "entity-widget-refresh"

    suspend fun assign(context: Context, glanceId: GlanceId, entity: EntityUi) {
        updateAppWidgetState(context, glanceId) { preferences ->
            preferences[ENTITY_KEY] = entity.key
            preferences.remove(ERROR)
            WidgetModel.from(entity).write(preferences)
        }

        EntityWidget().update(context, glanceId)
    }

    suspend fun refreshAll(context: Context) {
        val ids = GlanceAppWidgetManager(context).getGlanceIds(EntityWidget::class.java)
        if (ids.isEmpty()) {
            return
        }

        val result = load(context)
        ids.forEach { refresh(context, it, result) }
    }

    fun schedule(context: Context) {
        val request = PeriodicWorkRequestBuilder<EntityWidgetRefreshWorker>(15, TimeUnit.MINUTES)
            .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
            .build()

        WorkManager.getInstance(context)
            .enqueueUniquePeriodicWork(REFRESH_WORK, ExistingPeriodicWorkPolicy.KEEP, request)
    }

    fun cancel(context: Context) {
        WorkManager.getInstance(context).cancelUniqueWork(REFRESH_WORK)
    }

    private suspend fun refresh(context: Context, glanceId: GlanceId, result: LoadResult) {
        updateAppWidgetState(context, glanceId) { preferences ->
            val key = preferences[ENTITY_KEY] ?: return@updateAppWidgetState

            when (result) {
                LoadResult.SignedOut -> preferences[ERROR] = "Sign in to Home Gateway"
                LoadResult.Failed -> preferences[ERROR] = "Offline"
                is LoadResult.Loaded -> {
                    val entity = result.snapshot.entities.firstOrNull { it.key == key }
                    if (entity == null) {
                        preferences[ERROR] = "Device not found"
                    } else {
                        preferences.remove(ERROR)
                        WidgetModel.from(entity).write(preferences)
                    }
                }
            }
        }

        EntityWidget().update(context, glanceId)
    }

    private suspend fun load(context: Context): LoadResult {
        if (!context.gateway.auth.signedIn.value) {
            return LoadResult.SignedOut
        }

        return try {
            LoadResult.Loaded(context.gateway.entities.current())
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Log.w(TAG, "failed to load entities for widgets", e)
            LoadResult.Failed
        }
    }

    private sealed interface LoadResult {
        data object SignedOut : LoadResult
        data object Failed : LoadResult
        data class Loaded(val snapshot: EntitySnapshot) : LoadResult
    }
}
