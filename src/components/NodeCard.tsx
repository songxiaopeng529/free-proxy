interface Props {
  name: string;
  type?: string;
  delay?: number;
  selected: boolean;
  onClick: () => void;
}

function latencyClass(delay?: number): string {
  if (delay == null || delay === 0) return "latency-timeout";
  if (delay < 200) return "latency-fast";
  if (delay < 500) return "latency-medium";
  return "latency-slow";
}

function latencyText(delay?: number): string {
  if (delay == null || delay === 0) return "---";
  return `${delay}ms`;
}

export default function NodeCard({ name, type, delay, selected, onClick }: Props) {
  return (
    <div className={`node-card ${selected ? "selected" : ""}`} onClick={onClick}>
      <span className="node-name">{name}</span>
      {type && <span className="node-type">{type.toUpperCase()}</span>}
      <span className={`node-latency ${latencyClass(delay)}`}>
        {latencyText(delay)}
      </span>
    </div>
  );
}
