import {
  MIHOMO_API_BASE,
  MIHOMO_WS_BASE,
  DELAY_TEST_URL,
  DELAY_TEST_TIMEOUT,
} from "./constants";

let secret = "";

export function setSecret(s: string) {
  secret = s;
}

function headers(): Record<string, string> {
  const h: Record<string, string> = { "Content-Type": "application/json" };
  if (secret) h["Authorization"] = `Bearer ${secret}`;
  return h;
}

export const mihomoApi = {
  getVersion: () =>
    fetch(`${MIHOMO_API_BASE}/version`, { headers: headers() }).then((r) =>
      r.json(),
    ),

  getProxies: () =>
    fetch(`${MIHOMO_API_BASE}/proxies`, { headers: headers() }).then((r) =>
      r.json(),
    ),

  selectProxy: (group: string, name: string) =>
    fetch(`${MIHOMO_API_BASE}/proxies/${encodeURIComponent(group)}`, {
      method: "PUT",
      headers: headers(),
      body: JSON.stringify({ name }),
    }),

  testGroupDelay: (group: string) =>
    fetch(
      `${MIHOMO_API_BASE}/group/${encodeURIComponent(group)}/delay?url=${encodeURIComponent(DELAY_TEST_URL)}&timeout=${DELAY_TEST_TIMEOUT}`,
      { headers: headers() },
    ).then((r) => r.json()),

  testProxyDelay: (name: string) =>
    fetch(
      `${MIHOMO_API_BASE}/proxies/${encodeURIComponent(name)}/delay?url=${encodeURIComponent(DELAY_TEST_URL)}&timeout=${DELAY_TEST_TIMEOUT}`,
      { headers: headers() },
    ).then((r) => r.json()),

  patchConfig: (patch: Record<string, unknown>) =>
    fetch(`${MIHOMO_API_BASE}/configs`, {
      method: "PATCH",
      headers: headers(),
      body: JSON.stringify(patch),
    }),

  trafficStream: () =>
    new WebSocket(
      `${MIHOMO_WS_BASE}/traffic${secret ? `?token=${secret}` : ""}`,
    ),

  memoryStream: () =>
    new WebSocket(
      `${MIHOMO_WS_BASE}/memory${secret ? `?token=${secret}` : ""}`,
    ),
};
