import { useState, useCallback } from "react";
import { useProxyStore } from "../store/proxyStore";
import NodeCard from "./NodeCard";
import RuleEditor from "./RuleEditor";

export default function NodeList() {
  const { groups, enabled, selectNode, testGroupDelay } = useProxyStore();
  const [delays, setDelays] = useState<Record<string, number>>({});
  const [testing, setTesting] = useState(false);
  const [showRules, setShowRules] = useState(false);

  const primaryGroup = groups.find(
    (g) => g.name === "PROXY" || g.name === "proxy",
  ) || groups[0];

  const handleTestAll = useCallback(async () => {
    if (!primaryGroup || testing) return;
    setTesting(true);
    try {
      const result = await testGroupDelay(primaryGroup.name);
      setDelays((prev) => ({ ...prev, ...result }));
    } finally {
      setTesting(false);
    }
  }, [primaryGroup, testing, testGroupDelay]);

  if (!enabled) {
    return (
      <div className="card node-list">
        <div className="empty-state">Enable proxy to see nodes</div>
      </div>
    );
  }

  if (!primaryGroup) {
    return (
      <div className="card node-list">
        <div className="empty-state">No proxy groups found. Add a subscription first.</div>
      </div>
    );
  }

  return (
    <>
      <div className="card node-list">
        <div className="node-list-header">
          <span className="card-title">Nodes ({primaryGroup.all.length})</span>
          <div style={{ display: "flex", gap: 4 }}>
            <button
              className="btn btn-ghost"
              onClick={() => setShowRules(true)}
            >
              Rules
            </button>
            <button
              className="btn btn-ghost"
              onClick={handleTestAll}
              disabled={testing}
            >
              {testing ? "Testing..." : "Test All"}
            </button>
          </div>
        </div>
        <div className="node-list-content">
          {primaryGroup.all.map((name) => (
            <NodeCard
              key={name}
              name={name}
              delay={delays[name]}
              selected={name === primaryGroup.now}
              onClick={() => selectNode(primaryGroup.name, name)}
            />
          ))}
        </div>
      </div>
      {showRules && <RuleEditor onClose={() => setShowRules(false)} />}
    </>
  );
}
