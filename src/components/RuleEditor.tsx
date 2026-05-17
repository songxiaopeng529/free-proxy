import { useState, useEffect } from "react";
import { commands } from "../services/tauriCommands";
import { useProxyStore } from "../store/proxyStore";

const DEFAULT_RULES = [
  "GEOIP,CN,DIRECT",
  "GEOSITE,cn,DIRECT",
  "GEOSITE,geolocation-!cn,PROXY",
  "MATCH,PROXY",
].join("\n");

export default function RuleEditor({ onClose }: { onClose: () => void }) {
  const [rules, setRules] = useState("");
  const [saving, setSaving] = useState(false);
  const restartProxy = useProxyStore((s) => s.fetchNodes);

  useEffect(() => {
    commands
      .getRules()
      .then((r) => setRules(r.join("\n")))
      .catch(() => setRules(DEFAULT_RULES));
  }, []);

  const handleSave = async () => {
    setSaving(true);
    try {
      const ruleList = rules
        .split("\n")
        .map((r) => r.trim())
        .filter((r) => r && !r.startsWith("#"));
      await commands.setCustomRules(ruleList);
      await restartProxy();
      onClose();
    } catch (e) {
      console.error("save rules error:", e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="rule-editor-overlay" onClick={onClose}>
      <div className="rule-editor" onClick={(e) => e.stopPropagation()}>
        <h3>Custom Rules</h3>
        <textarea
          value={rules}
          onChange={(e) => setRules(e.target.value)}
          placeholder="GEOIP,CN,DIRECT&#10;MATCH,PROXY"
          spellCheck={false}
        />
        <div className="rule-editor-actions">
          <button className="btn btn-ghost" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? "Saving..." : "Save & Apply"}
          </button>
        </div>
      </div>
    </div>
  );
}
