package net.infk8s.homegateway.ui

import android.text.format.DateUtils
import java.time.Instant
import java.time.OffsetDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException

fun parseInstant(value: Any?): Instant? {
    val raw = value?.toString() ?: return null

    return try {
        OffsetDateTime.parse(raw).toInstant()
    } catch (e: DateTimeParseException) {
        null
    }
}

fun Instant.relative(): String = DateUtils.getRelativeTimeSpanString(
    toEpochMilli(),
    System.currentTimeMillis(),
    DateUtils.MINUTE_IN_MILLIS,
).toString()

fun Instant.clock(): String = CLOCK.format(atZone(ZoneId.systemDefault()))

private val CLOCK = DateTimeFormatter.ofPattern("HH:mm")
