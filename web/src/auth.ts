import { UserManager, WebStorageStateStore, type User } from "oidc-client-ts";

// Kanidm OIDC. `authority` matches `oauth.issuer` in the backend config and the
// issued access token's `aud` is `home-gateway`, which the backend validates.
const AUTHORITY = "https://idm.anurag.sh/oauth2/openid/home-gateway";
const CLIENT_ID = "home-gateway";

// The backend derives authorization from the `groups` claim, so `groups` must be
// requested alongside the standard OIDC scopes.
const SCOPE = "openid email profile groups";

// Local dev talks to a backend built with debug_assertions, which grants full
// access without a token, so we skip the Kanidm round-trip entirely.
export const AUTH_DISABLED = import.meta.env.DEV;

const userManager = AUTH_DISABLED
  ? null
  : new UserManager({
      authority: AUTHORITY,
      client_id: CLIENT_ID,
      redirect_uri: `${window.location.origin}/callback`,
      post_logout_redirect_uri: window.location.origin,
      response_type: "code",
      scope: SCOPE,
      automaticSilentRenew: true,
      userStore: new WebStorageStateStore({ store: window.localStorage }),
    });

let currentUser: User | null = null;

export function getAccessToken(): string | null {
  return currentUser?.access_token ?? null;
}

// Resolve the signed-in user, driving the redirect flow when needed. Returns
// once we hold a valid access token (or immediately when auth is disabled).
export async function ensureAuthenticated(): Promise<void> {
  if (!userManager) return;

  if (window.location.pathname === "/callback") {
    currentUser = await userManager.signinRedirectCallback();
    window.history.replaceState({}, "", "/");
    return;
  }

  currentUser = await userManager.getUser();
  if (!currentUser || currentUser.expired) {
    await userManager.signinRedirect();
    // signinRedirect navigates away; this promise never resolves.
    await new Promise(() => {});
  }

  userManager.events.addUserLoaded((user) => {
    currentUser = user;
  });
}
