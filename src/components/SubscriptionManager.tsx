import { useState } from "react";
import { useSubscriptionStore } from "../store/subscriptionStore";
import { useProxyStore } from "../store/proxyStore";
import SubscriptionItem from "./SubscriptionItem";

export default function SubscriptionManager() {
  const { subscriptions, loading, addSubscription, updateAll } =
    useSubscriptionStore();
  const fetchNodes = useProxyStore((s) => s.fetchNodes);
  const [url, setUrl] = useState("");
  const [adding, setAdding] = useState(false);

  const handleAdd = async () => {
    if (!url.trim()) return;
    setAdding(true);
    try {
      await addSubscription(url.trim());
      setUrl("");
      await fetchNodes();
    } catch (e) {
      console.error("add subscription error:", e);
    } finally {
      setAdding(false);
    }
  };

  const handleUpdateAll = async () => {
    await updateAll();
    await fetchNodes();
  };

  return (
    <div className="card sub-section">
      <div className="node-list-header">
        <span className="card-title">Subscriptions</span>
        {subscriptions.length > 0 && (
          <button
            className="btn btn-ghost"
            onClick={handleUpdateAll}
            disabled={loading}
          >
            {loading ? "Updating..." : "Update All"}
          </button>
        )}
      </div>

      {subscriptions.map((sub) => (
        <SubscriptionItem key={sub.id} subscription={sub} />
      ))}

      <div className="add-sub-form">
        <input
          placeholder="Subscription URL..."
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleAdd()}
        />
        <button
          className="btn btn-primary"
          onClick={handleAdd}
          disabled={adding || !url.trim()}
        >
          {adding ? "..." : "Add"}
        </button>
      </div>
    </div>
  );
}
