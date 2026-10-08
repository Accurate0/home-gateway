package net.infk8s.homegateway.quicktiles

import android.app.PendingIntent
import android.content.Intent
import android.graphics.drawable.Icon
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import android.util.Log
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.infk8s.homegateway.MainActivity
import net.infk8s.homegateway.R
import net.infk8s.homegateway.dashboard.EntityCommand
import net.infk8s.homegateway.dashboard.headline
import net.infk8s.homegateway.dashboard.icon
import net.infk8s.homegateway.dashboard.isActive
import net.infk8s.homegateway.dashboard.primaryCommand
import net.infk8s.homegateway.dashboard.status
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.widget.EntityWidgets

abstract class QuickActionTileService(private val slot: Int) : TileService() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

    private var entity: EntityUi? = null

    override fun onStartListening() {
        super.onStartListening()

        val assignment = gateway.quickTileAssignments.get(slot)
        if (assignment == null) {
            renderUnassigned()
            return
        }

        scope.launch {
            if (!gateway.auth.signedIn.value) {
                renderMessage(assignment, "Sign in")
                return@launch
            }

            try {
                val loaded = gateway.entities.current().entities.firstOrNull { it.key == assignment.key }
                entity = loaded

                if (loaded == null) {
                    renderMessage(assignment, "Not found")
                } else {
                    render(loaded)
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(TAG, "failed to load ${assignment.key}", e)
                renderMessage(assignment, "Offline")
            }
        }
    }

    override fun onClick() {
        super.onClick()

        val assignment = gateway.quickTileAssignments.get(slot)
        if (assignment == null) {
            open(pickerIntent())
            return
        }

        val command = entity?.primaryCommand()
        if (command == null) {
            open(Intent(this, MainActivity::class.java))
            return
        }

        val sensitive = command == EntityCommand.GARAGE_OPEN || command == EntityCommand.GARAGE_CLOSE
        if (sensitive && isLocked) {
            unlockAndRun { perform(assignment, command) }
        } else {
            perform(assignment, command)
        }
    }

    override fun onDestroy() {
        scope.cancel()
        super.onDestroy()
    }

    private fun perform(assignment: QuickTileAssignment, command: EntityCommand) {
        qsTile?.let { tile ->
            tile.state = when (command) {
                EntityCommand.LIGHT_ON, EntityCommand.GARAGE_OPEN, EntityCommand.PURIFIER_ON -> Tile.STATE_ACTIVE
                EntityCommand.LIGHT_OFF, EntityCommand.GARAGE_CLOSE, EntityCommand.PURIFIER_OFF -> Tile.STATE_INACTIVE
                EntityCommand.MEDIA_PLAY_PAUSE ->
                    if (tile.state == Tile.STATE_ACTIVE) Tile.STATE_INACTIVE else Tile.STATE_ACTIVE
            }
            tile.updateTile()
        }

        val context = applicationContext
        gateway.scope.launch {
            try {
                if (!command.execute(context.gateway.apollo, assignment.id)) {
                    Log.w(TAG, "$command on ${assignment.id} was rejected")
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(TAG, "$command on ${assignment.id} failed", e)
            }

            context.gateway.entities.invalidate()
            delay(SETTLE_MS)
            QuickTiles.refreshAll(context)
            EntityWidgets.refreshAll(context)
        }
    }

    private fun render(entity: EntityUi) {
        val tile = qsTile ?: return

        tile.label = entity.name
        tile.subtitle = entity.headline() ?: entity.status().replaceFirstChar { it.uppercase() }
        tile.icon = Icon.createWithResource(this, entity.icon())
        tile.state = if (entity.isActive()) Tile.STATE_ACTIVE else Tile.STATE_INACTIVE
        tile.updateTile()
    }

    private fun renderMessage(assignment: QuickTileAssignment, message: String) {
        val tile = qsTile ?: return

        tile.label = assignment.name
        tile.subtitle = message
        tile.state = Tile.STATE_INACTIVE
        tile.updateTile()
    }

    private fun renderUnassigned() {
        val tile = qsTile ?: return

        tile.label = getString(R.string.app_name)
        tile.subtitle = "Tap to set up"
        tile.icon = Icon.createWithResource(this, R.drawable.ic_nav_home)
        tile.state = Tile.STATE_INACTIVE
        tile.updateTile()
    }

    private fun pickerIntent() = Intent(this, QuickTileConfigureActivity::class.java)
        .putExtra(QuickTiles.EXTRA_SLOT, slot)

    private fun open(intent: Intent) {
        val pending = PendingIntent.getActivity(
            this,
            slot,
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )

        startActivityAndCollapse(pending)
    }

    private companion object {
        const val TAG = "QuickActionTile"
        const val SETTLE_MS = 1_500L
    }
}

class QuickTile1 : QuickActionTileService(0)

class QuickTile2 : QuickActionTileService(1)

class QuickTile3 : QuickActionTileService(2)

class QuickTile4 : QuickActionTileService(3)

class QuickTile5 : QuickActionTileService(4)
