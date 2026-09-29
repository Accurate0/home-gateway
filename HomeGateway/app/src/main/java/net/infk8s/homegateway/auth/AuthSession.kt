package net.infk8s.homegateway.auth

import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.core.content.edit
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.openid.appauth.AuthState
import net.openid.appauth.AuthorizationException
import net.openid.appauth.AuthorizationRequest
import net.openid.appauth.AuthorizationResponse
import net.openid.appauth.AuthorizationService
import net.openid.appauth.AuthorizationServiceConfiguration
import net.openid.appauth.ResponseTypeValues

class AuthSession(context: Context) {
    private val preferences = context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    private val service = AuthorizationService(context)

    private val mutex = Mutex()

    private var state = restore()

    private val _signedIn = MutableStateFlow(state.isAuthorized)
    val signedIn: StateFlow<Boolean> = _signedIn.asStateFlow()

    suspend fun authorizationIntent(): Intent {
        val configuration = discover()

        val request = AuthorizationRequest.Builder(
            configuration,
            AuthConfig.CLIENT_ID,
            ResponseTypeValues.CODE,
            AuthConfig.redirectUri,
        )
            .setScope(AuthConfig.SCOPE)
            .build()

        return service.getAuthorizationRequestIntent(request)
    }

    suspend fun completeAuthorization(intent: Intent) {
        val response = AuthorizationResponse.fromIntent(intent)
        val exception = AuthorizationException.fromIntent(intent)

        mutex.withLock {
            val next = AuthState(response, exception)
            if (response == null) {
                throw exception ?: IllegalStateException("sign-in was cancelled")
            }

            suspendCancellableCoroutine { continuation ->
                service.performTokenRequest(response.createTokenExchangeRequest()) { token, error ->
                    next.update(token, error)

                    if (token != null) {
                        continuation.resume(Unit)
                    } else {
                        continuation.resumeWithException(
                            error ?: IllegalStateException("token exchange failed"),
                        )
                    }
                }
            }

            replace(next)
        }
    }

    suspend fun freshToken(): String? = mutex.withLock {
        if (!state.isAuthorized) {
            return@withLock null
        }

        val current = state
        try {
            val token = suspendCancellableCoroutine { continuation ->
                current.performActionWithFreshTokens(service) { accessToken, _, error ->
                    if (accessToken != null) {
                        continuation.resume(accessToken)
                    } else {
                        continuation.resumeWithException(
                            error ?: IllegalStateException("no access token"),
                        )
                    }
                }
            }

            persist()
            token
        } catch (e: AuthorizationException) {
            if (e.type == AuthorizationException.TYPE_OAUTH_TOKEN_ERROR) {
                Log.w(TAG, "refresh token rejected, signing out", e)
                replace(AuthState())
                return@withLock null
            }

            throw e
        }
    }

    suspend fun signOut() {
        mutex.withLock { replace(AuthState()) }
    }

    private suspend fun discover(): AuthorizationServiceConfiguration =
        suspendCancellableCoroutine { continuation ->
            AuthorizationServiceConfiguration.fetchFromIssuer(AuthConfig.issuer) { configuration, error ->
                if (configuration != null) {
                    continuation.resume(configuration)
                } else {
                    continuation.resumeWithException(
                        error ?: IllegalStateException("oidc discovery failed"),
                    )
                }
            }
        }

    private fun replace(next: AuthState) {
        state = next
        persist()
        _signedIn.value = next.isAuthorized
    }

    private fun persist() {
        preferences.edit { putString(KEY_STATE, state.jsonSerializeString()) }
    }

    private fun restore(): AuthState {
        val raw = preferences.getString(KEY_STATE, null) ?: return AuthState()

        return try {
            AuthState.jsonDeserialize(raw)
        } catch (e: Exception) {
            Log.w(TAG, "discarding unreadable auth state", e)
            AuthState()
        }
    }

    private companion object {
        const val TAG = "AuthSession"
        const val PREFERENCES = "auth"
        const val KEY_STATE = "state"
    }
}
