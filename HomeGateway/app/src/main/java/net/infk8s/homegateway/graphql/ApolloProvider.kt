package net.infk8s.homegateway.graphql

import com.apollographql.apollo.ApolloClient
import com.apollographql.apollo.api.http.HttpRequest
import com.apollographql.apollo.api.http.HttpResponse
import com.apollographql.apollo.network.http.HttpInterceptor
import com.apollographql.apollo.network.http.HttpInterceptorChain
import com.apollographql.apollo.network.websocket.GraphQLWsProtocol
import com.apollographql.apollo.network.websocket.WebSocketNetworkTransport
import net.infk8s.homegateway.BuildConfig
import net.infk8s.homegateway.auth.AuthSession

/// Single shared Apollo client for the gateway GraphQL API.
/// URL convention mirrors PushTokenRegistrar: prod host vs the LAN dev box in debug.
object ApolloProvider {
    private const val PROD_HOST = "home.anurag.sh"
    private const val DEBUG_HOST = "192.168.0.104:8000"

    fun build(auth: AuthSession): ApolloClient {
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
                    auth.freshToken()?.let { mapOf("Authorization" to "Bearer $it") }
                },
            )
            .build()

        return ApolloClient.Builder()
            .httpServerUrl(httpUrl)
            .subscriptionNetworkTransport(wsTransport)
            .addHttpInterceptor(BearerInterceptor(auth))
            .build()
    }

    private class BearerInterceptor(private val auth: AuthSession) : HttpInterceptor {
        override suspend fun intercept(request: HttpRequest, chain: HttpInterceptorChain): HttpResponse {
            val token = auth.freshToken() ?: return chain.proceed(request)

            return chain.proceed(
                request.newBuilder().addHeader("Authorization", "Bearer $token").build(),
            )
        }
    }
}
