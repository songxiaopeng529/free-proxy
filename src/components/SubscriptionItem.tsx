import { useSubscriptionStore } from "../store/subscriptionStore";
import { useProxyStore } from "../store/proxyStore";
import type { Subscription } from "../types/subscription";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

function formatDate(ts: number | null): string {
  if (!ts) return "Never";
  return new Date(ts * 1000).toLocaleString();
}

export default function SubscriptionItem({ subscription: sub }: { subscription: Subscription }) {
  const { removeSubscription, updateSubscription } = useSubscriptionStore();
  const fetchNodes = useProxyStore((s) => s.fetchNodes);

  const handleUpdate = async () => {
    await updateSubscription(sub.id);
    await fetchNodes();
  };

  return (
    <div className="sub-item">
      <div className="sub-info">
        <div className="sub-name">{sub.name || sub.url}</div>
        <div className="sub-meta">
          {sub.node_count} nodes
          {sub.traffic_info && (
            <>
              {" "}
              &middot; {formatBytes(sub.traffic_info.download)} /{" "}
              {formatBytes(sub.traffic_info.total)}
            </>
          )}
          {" "}&middot; {formatDate(sub.last_updated)}
        </div>
      </div>
      <div className="sub-actions">
        <button className="icon-btn" onClick={handleUpdate} title="Update">
          &#x21bb;
        </button>
        <button
          className="icon-btn danger"
          onClick={() => removeSubscription(sub.id)}
          title="Remove"
        >
          &#x2715;
        </button>
      </div>
    </div>
  );
}
