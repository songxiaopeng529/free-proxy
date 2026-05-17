import { useEffect, useRef } from "react";
import { useProxyStore } from "../store/proxyStore";
import { useTrafficStore } from "../store/trafficStore";
import { mihomoApi } from "../services/mihomoApi";

function formatSpeed(bytes: number): string {
  if (bytes < 1024) return `${bytes} B/s`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB/s`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`;
}

export default function StatusBar() {
  const { coreStatus, enabled } = useProxyStore();
  const { upload, download, setTraffic } = useTrafficStore();
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (!enabled || coreStatus !== "running") {
      wsRef.current?.close();
      wsRef.current = null;
      setTraffic(0, 0);
      return;
    }

    const ws = mihomoApi.trafficStream();
    ws.onmessage = (e) => {
      try {
        const data = JSON.parse(e.data);
        setTraffic(data.up || 0, data.down || 0);
      } catch {
        // ignore
      }
    };
    ws.onerror = () => ws.close();
    wsRef.current = ws;
    return () => ws.close();
  }, [enabled, coreStatus, setTraffic]);

  return (
    <div className="status-bar">
      <div
        className={`status-dot ${coreStatus === "running" ? "running" : ""}`}
      />
      <span>
        {coreStatus === "running"
          ? "Connected"
          : coreStatus === "starting"
            ? "Starting..."
            : "Disconnected"}
      </span>
      <div className="traffic-info">
        <span className="traffic-up">&uarr; {formatSpeed(upload)}</span>
        <span className="traffic-down">&darr; {formatSpeed(download)}</span>
      </div>
    </div>
  );
}
