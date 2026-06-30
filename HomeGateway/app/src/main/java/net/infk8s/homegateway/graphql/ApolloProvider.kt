package net.infk8s.homegateway.graphql

import com.apollographql.apollo.ApolloClient
import com.apollographql.apollo.network.websocket.GraphQLWsProtocol
import com.apollographql.apollo.network.websocket.WebSocketNetworkTransport
import net.infk8s.homegateway.BuildConfig

/// Single shared Apollo client for the gateway GraphQL API.
/// URL convention mirrors PushTokenRegistrar: prod host vs the LAN dev box in debug.
object ApolloProvider {
    private const val PROD_HOST = "home.anurag.sh"
    private const val DEBUG_HOST = "192.168.0.104:8000"

    val client: ApolloClient by lazy { build() }

    private fun build(): ApolloClient {
        val httpUrl: String
        val wsUrl: String
        if (BuildConfig.DEBUG) {
            httpUrl = "http://$DEBUG_HOST/v1/graphql"
            wsUrl = "ws://$DEBUG_HOST/v1/graphql/ws"
        } else {
            httpUrl = "https://$PROD_HOST/v1/graphql"
            wsUrl = "wss://$PROD_HOST/v1/graphql/ws"
        }

        val wsTransport = WebSocketNetworkTransport.Builder()
            .serverUrl(wsUrl)
            .wsProtocol(
                GraphQLWsProtocol {
                    // The backend reads the WS auth token from the connection-init payload.
                    // Debug builds get full_access server-side, so no key is needed there.
                    if (BuildConfig.DEBUG) emptyMap()
                    else mapOf("X-Api-Key" to BuildConfig.HOME_GATEWAY_GRAPHQL_API_KEY)
                },
            )
            .build()

        val builder = ApolloClient.Builder()
            .httpServerUrl(httpUrl)
            .subscriptionNetworkTransport(wsTransport)

        // HTTP auth: debug builds need no key (server grants full_access).
        if (!BuildConfig.DEBUG) {
            builder.addHttpHeader("X-Api-Key", BuildConfig.HOME_GATEWAY_GRAPHQL_API_KEY)
        }

        return builder.build()
    }
}
