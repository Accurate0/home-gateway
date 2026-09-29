package net.infk8s.homegateway.auth

import android.net.Uri
import androidx.core.net.toUri

object AuthConfig {
    val issuer: Uri = "https://idm.anurag.sh/oauth2/openid/home-gateway".toUri()
    val redirectUri: Uri = "net.infk8s.homegateway:/oauth2redirect".toUri()

    const val CLIENT_ID = "home-gateway"
    const val SCOPE = "openid email profile groups"
}
