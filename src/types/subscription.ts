export interface Subscription {
  id: string;
  name: string;
  url: string;
  last_updated: number | null;
  node_count: number;
  traffic_info: TrafficInfo | null;
}

export interface TrafficInfo {
  upload: number;
  download: number;
  total: number;
  expire: number | null;
}
