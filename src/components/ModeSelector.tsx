import { useProxyStore } from "../store/proxyStore";
import type { ProxyMode } from "../types/proxy";

const modes: { value: ProxyMode; label: string }[] = [
  { value: "global", label: "Global" },
  { value: "rule", label: "Rule" },
  { value: "direct", label: "Direct" },
];

export default function ModeSelector() {
  const { mode, setMode } = useProxyStore();

  return (
    <div className="mode-selector">
      {modes.map((m) => (
        <button
          key={m.value}
          className={`mode-btn ${mode === m.value ? "active" : ""}`}
          onClick={() => setMode(m.value)}
        >
          {m.label}
        </button>
      ))}
    </div>
  );
}
