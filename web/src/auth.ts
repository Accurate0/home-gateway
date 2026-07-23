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
export const AUTH_DISABLED =
  import.meta.env.DEV || import.meta.env.VITE_AUTH_DISABLED === "true";

export function getApiKey(): string | null {
  return localStorage.getItem("apiKey");
}

export function setApiKey(value: string): void {
  localStorage.setItem("apiKey", value.trim());
}

// Persist both the signed-in user and the transient signin state (PKCE verifier
// + state) in localStorage so they survive the full-page redirect to Kanidm and
// back. Using the same store for both avoids "No matching state found" races.
const store = new WebStorageStateStore({ store: window.localStorage });

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
      userStore: store,
      stateStore: store,
    });

let currentUser: User | null = null;

export function getAccessToken(): string | null {
  return currentUser?.access_token ?? null;
}

async function login(): Promise<never> {
  await userManager!.signinRedirect();
  // signinRedirect navigates away; this promise never resolves.
  return new Promise<never>(() => {});
}

// Resolve the signed-in user, driving the redirect flow when needed. Returns
// once we hold a valid access token (or immediately when auth is disabled).
export async function ensureAuthenticated(): Promise<void> {
  if (!userManager) return;

  const params = new URLSearchParams(window.location.search);
  if (window.location.pathname === "/callback" && params.has("code")) {
    try {
      currentUser = await userManager.signinRedirectCallback();
    } catch (e) {
      // Strip the consumed code/state so a reload can't replay this callback,
      // then surface a clear message instead of an uncaught error + reload loop.
      window.history.replaceState({}, "", "/");
      throw new Error(
        `Sign-in failed during token exchange: ${
          e instanceof Error ? e.message : String(e)
        }`,
        { cause: e },
      );
    }
    window.history.replaceState({}, "", "/");
  }

  currentUser = currentUser ?? (await userManager.getUser());
  if (!currentUser || currentUser.expired) {
    await login();
  }

  userManager.events.addUserLoaded((user) => {
    currentUser = user;
  });
}
