package net.infk8s.homegateway.widget

import android.content.Context
import android.util.Log
import androidx.glance.GlanceId
import androidx.glance.action.ActionParameters
import androidx.glance.appwidget.action.ActionCallback
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import net.infk8s.homegateway.dashboard.EntityCommand
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.quicktiles.QuickTiles

class EntityWidgetAction : ActionCallback {
    override suspend fun onAction(context: Context, glanceId: GlanceId, parameters: ActionParameters) {
        val id = parameters[ID] ?: return
        val command = parameters[COMMAND]?.let { stored -> EntityCommand.entries.firstOrNull { it.name == stored } } ?: return

        try {
            if (!command.execute(context.gateway.apollo, id)) {
                Log.w(TAG, "$command on $id was rejected")
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Log.w(TAG, "$command on $id failed", e)
        }

        context.gateway.entities.invalidate()
        delay(SETTLE_MS)
        EntityWidgets.refreshAll(context)
        QuickTiles.refreshAll(context)
    }

    companion object {
        val ID = ActionParameters.Key<String>("id")
        val COMMAND = ActionParameters.Key<String>("command")

        private const val TAG = "EntityWidgetAction"
        private const val SETTLE_MS = 1_500L
    }
}
