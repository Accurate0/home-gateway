package net.infk8s.homegateway.auth

import android.net.Uri

object AuthConfig {
    val issuer: Uri = Uri.parse("https://idm.anurag.sh/oauth2/openid/home-gateway")
    val redirectUri: Uri = Uri.parse("net.infk8s.homegateway:/oauth2redirect")

    const val CLIENT_ID = "home-gateway"
    const val SCOPE = "openid email profile groups"
}
