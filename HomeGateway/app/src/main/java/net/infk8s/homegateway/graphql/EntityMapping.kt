package net.infk8s.homegateway.graphql

import net.infk8s.homegateway.graphql.type.HomeCapability
import net.infk8s.homegateway.ui.parseInstant

fun EntitiesQuery.HomeEntity.toUi(): EntityUi? = when {
    onHomeLightEntity != null ->
        EntityUi.Light(
            id = onHomeLightEntity.id,
            name = onHomeLightEntity.name,
            room = onHomeLightEntity.room,
            category = onHomeLightEntity.category,
            on = onHomeLightEntity.on,
            dimmable = onHomeLightEntity.capabilities.any { it == HomeCapability.BRIGHTNESS },
            tunable = onHomeLightEntity.capabilities.any { it == HomeCapability.COLOUR_TEMP },
            colour = onHomeLightEntity.capabilities.any { it == HomeCapability.RGB },
        )

    onHomeDoorEntity != null ->
        EntityUi.Door(
            id = onHomeDoorEntity.id,
            name = onHomeDoorEntity.name,
            room = onHomeDoorEntity.room,
            category = onHomeDoorEntity.category,
            open = onHomeDoorEntity.open,
        )

    onHomeGarageDoorEntity != null ->
        EntityUi.GarageDoor(
            id = onHomeGarageDoorEntity.id,
            name = onHomeGarageDoorEntity.name,
            room = onHomeGarageDoorEntity.room,
            category = onHomeGarageDoorEntity.category,
            state = onHomeGarageDoorEntity.garageState,
            batteryPercentage = onHomeGarageDoorEntity.battery?.percentage,
        )

    onHomeAirPurifierEntity != null ->
        EntityUi.AirPurifier(
            id = onHomeAirPurifierEntity.id,
            name = onHomeAirPurifierEntity.name,
            room = onHomeAirPurifierEntity.room,
            category = onHomeAirPurifierEntity.category,
            on = onHomeAirPurifierEntity.on,
            mode = onHomeAirPurifierEntity.purifierMode,
            speed = onHomeAirPurifierEntity.purifierSpeed,
            pm25 = onHomeAirPurifierEntity.purifierPm25,
            aqi = onHomeAirPurifierEntity.aqi,
            cadr = onHomeAirPurifierEntity.cadr,
            filterLife = onHomeAirPurifierEntity.filterLife,
            displayOn = onHomeAirPurifierEntity.displayOn,
            lock = onHomeLockableEntity?.lock?.let { DeviceLockUi(it.source, parseInstant(it.expiresAt)) },
        )

    onHomePresenceEntity != null ->
        EntityUi.Presence(
            id = onHomePresenceEntity.id,
            name = onHomePresenceEntity.name,
            room = onHomePresenceEntity.room,
            category = onHomePresenceEntity.category,
            present = onHomePresenceEntity.present,
        )

    onHomeEnvironmentEntity != null ->
        EntityUi.Environment(
            id = onHomeEnvironmentEntity.id,
            name = onHomeEnvironmentEntity.name,
            room = onHomeEnvironmentEntity.room,
            category = onHomeEnvironmentEntity.category,
            temperature = onHomeEnvironmentEntity.temperature,
            humidity = onHomeEnvironmentEntity.humidity,
            pressure = onHomeEnvironmentEntity.pressure,
            lux = onHomeEnvironmentEntity.lux,
            uvIndex = onHomeEnvironmentEntity.uvIndex,
            pm25 = onHomeEnvironmentEntity.pm25,
            vocIndex = onHomeEnvironmentEntity.vocIndex,
            lastSeen = parseInstant(onHomeEnvironmentEntity.lastSeen),
        )

    onHomePlantEntity != null ->
        EntityUi.Plant(
            id = onHomePlantEntity.id,
            name = onHomePlantEntity.name,
            room = onHomePlantEntity.room,
            category = onHomePlantEntity.category,
            soilMoisture = onHomePlantEntity.soilMoisture,
            batteryPercentage = onHomePlantEntity.battery?.percentage,
        )

    onHomeEinkDisplayEntity != null ->
        EntityUi.EinkDisplay(
            id = onHomeEinkDisplayEntity.id,
            name = onHomeEinkDisplayEntity.name,
            room = onHomeEinkDisplayEntity.room,
            category = onHomeEinkDisplayEntity.category,
            batteryPercentage = onHomeEinkDisplayEntity.batteryPercentage,
            isCharging = onHomeEinkDisplayEntity.isCharging,
        )

    onHomeRobotVacuumEntity != null ->
        EntityUi.RobotVacuum(
            id = onHomeRobotVacuumEntity.id,
            name = onHomeRobotVacuumEntity.name,
            room = onHomeRobotVacuumEntity.room,
            category = onHomeRobotVacuumEntity.category,
            status = onHomeRobotVacuumEntity.status,
            batteryPercentage = onHomeRobotVacuumEntity.batteryPercentage,
            currentRoom = onHomeRobotVacuumEntity.currentRoom,
        )

    onHomeMediaPlayerEntity != null ->
        EntityUi.MediaPlayer(
            id = onHomeMediaPlayerEntity.id,
            name = onHomeMediaPlayerEntity.name,
            room = onHomeMediaPlayerEntity.room,
            category = onHomeMediaPlayerEntity.category,
            playing = onHomeMediaPlayerEntity.playing,
            appName = onHomeMediaPlayerEntity.appName,
            mediaTitle = onHomeMediaPlayerEntity.mediaTitle,
            mediaSeriesTitle = onHomeMediaPlayerEntity.mediaSeriesTitle,
        )

    else -> null
}
