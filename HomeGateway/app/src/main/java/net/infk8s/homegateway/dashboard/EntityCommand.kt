package net.infk8s.homegateway.dashboard

import com.apollographql.apollo.ApolloClient
import com.apollographql.apollo.api.Mutation
import net.infk8s.homegateway.graphql.GarageDoorCloseMutation
import net.infk8s.homegateway.graphql.GarageDoorOpenMutation
import net.infk8s.homegateway.graphql.LightOffMutation
import net.infk8s.homegateway.graphql.LightOnMutation
import net.infk8s.homegateway.graphql.MediaPlayPauseMutation

enum class EntityCommand(val label: String) {
    LIGHT_ON("Turn on"),
    LIGHT_OFF("Turn off"),
    GARAGE_OPEN("Open"),
    GARAGE_CLOSE("Close"),
    MEDIA_PLAY_PAUSE("Play/Pause");

    fun mutation(id: String): Mutation<*> = when (this) {
        LIGHT_ON -> LightOnMutation(id)
        LIGHT_OFF -> LightOffMutation(id)
        GARAGE_OPEN -> GarageDoorOpenMutation(id)
        GARAGE_CLOSE -> GarageDoorCloseMutation(id)
        MEDIA_PLAY_PAUSE -> MediaPlayPauseMutation(id)
    }

    suspend fun execute(apollo: ApolloClient, id: String): Boolean {
        val response = apollo.mutation(mutation(id)).execute()

        return response.data != null && !response.hasErrors()
    }
}
