package net.infk8s.homegateway.dashboard

import androidx.annotation.DrawableRes
import net.infk8s.homegateway.R
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.graphql.type.GarageDoorState
import net.infk8s.homegateway.ui.theme.StateTone

data class TilePill(val label: String, val active: Boolean?)

fun EntityUi.activeTone(): StateTone? = when (this) {
    is EntityUi.Light -> StateTone.LIGHT.takeIf { on == true }
    is EntityUi.Door -> StateTone.OPEN.takeIf { open == true }
    is EntityUi.GarageDoor -> StateTone.OPEN.takeIf { state != null && state != GarageDoorState.CLOSED }
    is EntityUi.Presence -> StateTone.PRESENT.takeIf { present == true }
    is EntityUi.Environment, is EntityUi.EinkDisplay, is EntityUi.RobotVacuum, is EntityUi.MediaPlayer -> null
}

@DrawableRes
fun EntityUi.icon(): Int = when (this) {
    is EntityUi.Light -> if (on == true) R.drawable.ic_light_on else R.drawable.ic_light_off
    is EntityUi.Door -> if (open == true) R.drawable.ic_door_open else R.drawable.ic_door_closed
    is EntityUi.GarageDoor -> R.drawable.ic_garage
    is EntityUi.Presence -> if (present == true) R.drawable.ic_person else R.drawable.ic_person_away
    is EntityUi.Environment -> R.drawable.ic_thermometer
    is EntityUi.EinkDisplay -> R.drawable.ic_display
    is EntityUi.RobotVacuum -> R.drawable.ic_robot
    is EntityUi.MediaPlayer -> R.drawable.ic_tv
}

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
    is EntityUi.Environment -> null
}

fun EntityUi.battery(): Double? = when (this) {
    is EntityUi.GarageDoor -> batteryPercentage
    is EntityUi.EinkDisplay -> batteryPercentage
    is EntityUi.RobotVacuum -> batteryPercentage
    is EntityUi.Light, is EntityUi.Door, is EntityUi.Presence,
    is EntityUi.Environment, is EntityUi.MediaPlayer -> null
}

fun EntityUi.subtitle(): String = when (this) {
    is EntityUi.MediaPlayer -> listOfNotNull(mediaSeriesTitle, mediaTitle, appName).firstOrNull()
    is EntityUi.RobotVacuum -> currentRoom
    else -> null
} ?: room ?: id

fun EntityUi.mainAction(controls: EntityControls): (() -> Unit)? = when (this) {
    is EntityUi.Light -> {
        { controls.setLight(id, on != true) }
    }

    is EntityUi.GarageDoor -> when (state) {
        GarageDoorState.OPEN, GarageDoorState.OPENING -> {
            { controls.garageDoorClose(id) }
        }

        GarageDoorState.CLOSED, GarageDoorState.CLOSING -> {
            { controls.garageDoorOpen(id) }
        }

        else -> null
    }

    is EntityUi.MediaPlayer -> {
        { controls.mediaPlayPause(id) }
    }

    is EntityUi.Door, is EntityUi.Presence, is EntityUi.Environment,
    is EntityUi.EinkDisplay, is EntityUi.RobotVacuum -> null
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

private fun Boolean?.toPill(on: String, off: String): TilePill = when (this) {
    true -> TilePill(on, true)
    false -> TilePill(off, false)
    null -> TilePill("unknown", null)
}
