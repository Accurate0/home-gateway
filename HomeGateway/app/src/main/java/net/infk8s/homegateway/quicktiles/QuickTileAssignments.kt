package net.infk8s.homegateway.quicktiles

import android.content.Context
import androidx.core.content.edit
import net.infk8s.homegateway.graphql.EntityUi

data class QuickTileAssignment(val key: String, val id: String, val name: String)

class QuickTileAssignments(context: Context) {
    private val preferences = context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    fun get(slot: Int): QuickTileAssignment? {
        val key = preferences.getString(keyOf(slot), null) ?: return null
        val id = preferences.getString(idOf(slot), null) ?: return null

        return QuickTileAssignment(key, id, preferences.getString(nameOf(slot), null) ?: id)
    }

    fun set(slot: Int, entity: EntityUi) {
        preferences.edit {
            putString(keyOf(slot), entity.key)
            putString(idOf(slot), entity.id)
            putString(nameOf(slot), entity.name)
        }
    }

    private fun keyOf(slot: Int) = "slot_${slot}_key"

    private fun idOf(slot: Int) = "slot_${slot}_id"

    private fun nameOf(slot: Int) = "slot_${slot}_name"

    private companion object {
        const val PREFERENCES = "quick_tiles"
    }
}
