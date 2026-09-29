package net.infk8s.homegateway.dashboard

import androidx.annotation.DrawableRes
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.graphql.type.GarageDoorState
import net.infk8s.homegateway.ui.theme.StateTone

data class TilePill(val label: String, val active: Boolean?)

fun EntityUi.activeTone(): StateTone? = when (this) {
    is EntityUi.Light -> StateTone.LIGHT.takeIf { on == true }
    is EntityUi.Door -> StateTone.OPEN.takeIf { open == true }
    is EntityUi.GarageDoor -> StateTone.OPEN.takeIf { state != null && state != GarageDoorState.CLOSED }
    is EntityUi.Presence -> StateTone.PRESENT.takeIf { present == true }
    is EntityUi.Environment, is EntityUi.Plant, is EntityUi.EinkDisplay,
    is EntityUi.RobotVacuum, is EntityUi.MediaPlayer -> null
}

fun EntityUi.Plant.moistureStatus(): String? = soilMoisture?.let {
    when {
        it < PLANT_DRY -> "dry"
        it > PLANT_WET -> "overwatered"
        else -> "ok"
    }
}

fun EntityUi.Plant.moistureTone(): StateTone? = soilMoisture?.let {
    if (it < PLANT_DRY || it > PLANT_WET) StateTone.OPEN else StateTone.PRESENT
}

fun EntityUi.glyph(): EntityGlyph = when (this) {
    is EntityUi.Light -> if (on == true) EntityGlyph.LIGHT_ON else EntityGlyph.LIGHT_OFF
    is EntityUi.Door -> if (open == true) EntityGlyph.DOOR_OPEN else EntityGlyph.DOOR_CLOSED
    is EntityUi.GarageDoor -> EntityGlyph.GARAGE
    is EntityUi.Presence -> if (present == true) EntityGlyph.PERSON else EntityGlyph.PERSON_AWAY
    is EntityUi.Environment -> EntityGlyph.THERMOMETER
    is EntityUi.Plant -> EntityGlyph.PLANT
    is EntityUi.EinkDisplay -> EntityGlyph.DISPLAY
    is EntityUi.RobotVacuum -> EntityGlyph.ROBOT
    is EntityUi.MediaPlayer -> EntityGlyph.TV
}

@DrawableRes
fun EntityUi.icon(): Int = glyph().drawable

fun EntityUi.isActive(): Boolean = activeTone() != null || (this is EntityUi.MediaPlayer && playing)

fun EntityUi.headline(): String? = when (this) {
    is EntityUi.Environment -> temperature?.let { "%.1f°".format(it) }
    is EntityUi.Plant -> soilMoisture?.let { "%.0f%%".format(it) }
    else -> null
}

fun EntityUi.status(): String = pill()?.label ?: details()

fun EntityUi.pill(): TilePill? = when (this) {
    is EntityUi.Light -> on.toPill("on", "off")
    is EntityUi.Door -> open.toPill("open", "closed")
    is EntityUi.Presence -> present.toPill("present", "away")
    is EntityUi.GarageDoor -> TilePill(
        state?.rawValue?.lowercase() ?: "unknown",
        state?.let { it != GarageDoorState.CLOSED },
    )
    is EntityUi.MediaPlayer -> TilePill(if (playing) "playing" else "paused", playing)
    is EntityUi.RobotVacuum -> TilePill(status ?: "unknown", status?.let { true })
    is EntityUi.EinkDisplay -> if (isCharging == true) TilePill("charging", true) else null
    is EntityUi.Environment, is EntityUi.Plant -> null
}

fun EntityUi.battery(): Double? = when (this) {
    is EntityUi.GarageDoor -> batteryPercentage
    is EntityUi.EinkDisplay -> batteryPercentage
    is EntityUi.RobotVacuum -> batteryPercentage
    is EntityUi.Plant -> batteryPercentage
    is EntityUi.Light, is EntityUi.Door, is EntityUi.Presence,
    is EntityUi.Environment, is EntityUi.MediaPlayer -> null
}

fun EntityUi.subtitle(): String = when (this) {
    is EntityUi.MediaPlayer -> listOfNotNull(mediaSeriesTitle, mediaTitle, appName).firstOrNull()
    is EntityUi.RobotVacuum -> currentRoom
    else -> null
} ?: room ?: id

fun EntityUi.primaryCommand(): EntityCommand? = when (this) {
    is EntityUi.Light -> if (on == true) EntityCommand.LIGHT_OFF else EntityCommand.LIGHT_ON

    is EntityUi.GarageDoor -> when (state) {
        GarageDoorState.OPEN, GarageDoorState.OPENING -> EntityCommand.GARAGE_CLOSE
        GarageDoorState.CLOSED, GarageDoorState.CLOSING -> EntityCommand.GARAGE_OPEN
        else -> null
    }

    is EntityUi.MediaPlayer -> EntityCommand.MEDIA_PLAY_PAUSE

    is EntityUi.Door, is EntityUi.Presence, is EntityUi.Environment, is EntityUi.Plant,
    is EntityUi.EinkDisplay, is EntityUi.RobotVacuum -> null
}

fun EntityUi.mainAction(controls: EntityControls): (() -> Unit)? {
    val command = primaryCommand() ?: return null

    return { controls.run(id, command) }
}

fun EntityUi.details(): String = when (this) {
    is EntityUi.Light -> on?.let { if (it) "On" else "Off" } ?: "Unknown"
    is EntityUi.Door -> open?.let { if (it) "Open" else "Closed" } ?: "Unknown"
    is EntityUi.GarageDoor -> buildList {
        add(state?.rawValue?.lowercase()?.replaceFirstChar { it.uppercase() } ?: "Unknown")
        batteryPercentage?.let { add("%.0f%%".format(it)) }
    }.joinToString(" · ")
    is EntityUi.Presence -> present?.let { if (it) "Present" else "Away" } ?: "Unknown"
    is EntityUi.Environment -> buildList {
        temperature?.let { add("%.1f°C".format(it)) }
        humidity?.let { add("%.0f%%".format(it)) }
    }.joinToString(" · ").ifEmpty { "Unknown" }

    is EntityUi.Plant -> buildList {
        soilMoisture?.let { add("%.0f%% moisture".format(it)) }
        moistureStatus()?.let { add(it.replaceFirstChar { c -> c.uppercase() }) }
    }.joinToString(" · ").ifEmpty { "Unknown" }

    is EntityUi.EinkDisplay -> buildList {
        batteryPercentage?.let { add("%.0f%%".format(it)) }
        if (isCharging == true) add("Charging")
    }.joinToString(" · ").ifEmpty { "Unknown" }

    is EntityUi.RobotVacuum -> buildList {
        status?.let { add(it) }
        currentRoom?.let { add(it) }
        batteryPercentage?.let { add("%.0f%%".format(it)) }
    }.joinToString(" · ").ifEmpty { "Unknown" }

    is EntityUi.MediaPlayer -> buildList {
        add(if (playing) "Playing" else "Paused")
        appName?.let { add(it) }
        listOfNotNull(mediaSeriesTitle, mediaTitle).firstOrNull()?.let { add(it) }
    }.joinToString(" · ")
}

private const val PLANT_DRY = 20.0
private const val PLANT_WET = 80.0

private fun Boolean?.toPill(on: String, off: String): TilePill = when (this) {
    true -> TilePill(on, true)
    false -> TilePill(off, false)
    null -> TilePill("unknown", null)
}
