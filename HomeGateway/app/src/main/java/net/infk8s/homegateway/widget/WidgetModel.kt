package net.infk8s.homegateway.widget

import androidx.datastore.preferences.core.MutablePreferences
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.stringPreferencesKey
import net.infk8s.homegateway.dashboard.EntityCommand
import net.infk8s.homegateway.dashboard.EntityGlyph
import net.infk8s.homegateway.dashboard.activeTone
import net.infk8s.homegateway.dashboard.battery
import net.infk8s.homegateway.dashboard.glyph
import net.infk8s.homegateway.dashboard.headline
import net.infk8s.homegateway.dashboard.primaryCommand
import net.infk8s.homegateway.dashboard.status
import net.infk8s.homegateway.dashboard.subtitle
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.ui.theme.StateTone

data class WidgetModel(
    val id: String,
    val name: String,
    val subtitle: String,
    val headline: String?,
    val status: String,
    val tone: StateTone?,
    val glyph: EntityGlyph,
    val battery: String?,
    val command: EntityCommand?,
) {
    fun write(preferences: MutablePreferences) {
        preferences[ID] = id
        preferences[NAME] = name
        preferences[SUBTITLE] = subtitle
        preferences.putOrRemove(HEADLINE, headline)
        preferences[STATUS] = status
        preferences.putOrRemove(TONE, tone?.name)
        preferences[GLYPH] = glyph.name
        preferences.putOrRemove(BATTERY, battery)
        preferences.putOrRemove(COMMAND, command?.name)
    }

    companion object {
        private val ID = stringPreferencesKey("model_id")
        private val NAME = stringPreferencesKey("model_name")
        private val SUBTITLE = stringPreferencesKey("model_subtitle")
        private val HEADLINE = stringPreferencesKey("model_headline")
        private val STATUS = stringPreferencesKey("model_status")
        private val TONE = stringPreferencesKey("model_tone")
        private val GLYPH = stringPreferencesKey("model_glyph")
        private val BATTERY = stringPreferencesKey("model_battery")
        private val COMMAND = stringPreferencesKey("model_command")

        fun from(entity: EntityUi) = WidgetModel(
            id = entity.id,
            name = entity.name,
            subtitle = entity.subtitle(),
            headline = entity.headline(),
            status = entity.status().replaceFirstChar { it.uppercase() },
            tone = entity.activeTone(),
            glyph = entity.glyph(),
            battery = entity.battery()?.let { "%.0f%%".format(it) },
            command = entity.primaryCommand(),
        )

        fun read(preferences: Preferences): WidgetModel? {
            val id = preferences[ID] ?: return null
            val name = preferences[NAME] ?: return null

            return WidgetModel(
                id = id,
                name = name,
                subtitle = preferences[SUBTITLE].orEmpty(),
                headline = preferences[HEADLINE],
                status = preferences[STATUS].orEmpty(),
                tone = preferences[TONE]?.let { stored -> StateTone.entries.firstOrNull { it.name == stored } },
                glyph = preferences[GLYPH]
                    ?.let { stored -> EntityGlyph.entries.firstOrNull { it.name == stored } }
                    ?: EntityGlyph.DISPLAY,
                battery = preferences[BATTERY],
                command = preferences[COMMAND]?.let { stored -> EntityCommand.entries.firstOrNull { it.name == stored } },
            )
        }

        private fun MutablePreferences.putOrRemove(key: Preferences.Key<String>, value: String?) {
            if (value == null) remove(key) else set(key, value)
        }
    }
}
