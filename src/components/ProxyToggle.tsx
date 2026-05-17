import { useProxyStore } from "../store/proxyStore";

export default function ProxyToggle() {
  const { enabled, loading, toggleProxy } = useProxyStore();

  return (
    <div
      className={`toggle ${enabled ? "active" : ""}`}
      onClick={loading ? undefined : toggleProxy}
      style={{ opacity: loading ? 0.5 : 1 }}
    />
  );
}
