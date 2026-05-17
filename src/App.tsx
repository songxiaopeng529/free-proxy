import { useEffect } from "react";
import "./App.css";
import ProxyToggle from "./components/ProxyToggle";
import ModeSelector from "./components/ModeSelector";
import NodeList from "./components/NodeList";
import SubscriptionManager from "./components/SubscriptionManager";
import StatusBar from "./components/StatusBar";
import { useProxyStore } from "./store/proxyStore";
import { useSubscriptionStore } from "./store/subscriptionStore";

function App() {
  const fetchStatus = useProxyStore((s) => s.fetchStatus);
  const fetchSubscriptions = useSubscriptionStore((s) => s.fetchSubscriptions);

  useEffect(() => {
    fetchStatus();
    fetchSubscriptions();
  }, [fetchStatus, fetchSubscriptions]);

  return (
    <div className="app">
      <header className="app-header">
        <h1>Free Proxy</h1>
        <ProxyToggle />
      </header>
      <ModeSelector />
      <NodeList />
      <SubscriptionManager />
      <StatusBar />
    </div>
  );
}

export default App;
