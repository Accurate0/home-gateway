package net.infk8s.homegateway.quicktiles

import android.content.ComponentName
import android.content.Context
import android.service.quicksettings.TileService
import net.infk8s.homegateway.gateway

object QuickTiles {
    const val EXTRA_SLOT = "net.infk8s.homegateway.quicktiles.SLOT"

    private val services = listOf(
        QuickTile1::class.java,
        QuickTile2::class.java,
        QuickTile3::class.java,
        QuickTile4::class.java,
        QuickTile5::class.java,
    )

    fun slotOf(component: ComponentName): Int? =
        services.indexOfFirst { it.name == component.className }.takeIf { it >= 0 }

    fun refresh(context: Context, slot: Int) {
        val service = services.getOrNull(slot) ?: return

        TileService.requestListeningState(context, ComponentName(context, service))
    }

    fun refreshAll(context: Context) {
        val assignments = context.gateway.quickTileAssignments

        services.indices
            .filter { assignments.get(it) != null }
            .forEach { refresh(context, it) }
    }
}
