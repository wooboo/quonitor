import { useState } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from "recharts";
import { useHistoricalData, useModelUsageHistory } from "../hooks/useQuotaData";

interface TrendChartProps {
  accountId: string;
  days?: number;
}

export default function TrendChart({ accountId, days = 7 }: TrendChartProps) {
  const [view, setView] = useState<"account" | "model">("account");
  const { data: snapshots = [], isLoading: snapshotsLoading } = useHistoricalData(accountId, days);
  const { data: modelUsage = [], isLoading: modelLoading } = useModelUsageHistory(accountId, days);

  if (snapshotsLoading || modelLoading) {
    return <div className="text-center text-gray-400 py-4">Loading chart...</div>;
  }

  if (view === "account") {
    if (!snapshots || snapshots.length === 0) {
      return <div className="text-center text-gray-400 py-4">No historical data available</div>;
    }

    const chartData = snapshots.map((snapshot: any) => ({
      date: new Date(snapshot.timestamp * 1000).toLocaleDateString(),
      cost: snapshot.cost_usd || 0,
      input: (snapshot.tokens_input || 0) / 1000000,
      output: (snapshot.tokens_output || 0) / 1000000,
    }));

    return (
      <div className="space-y-3">
        <div className="flex gap-2">
          <button
            onClick={() => setView("account")}
            className={`px-3 py-1 text-sm rounded ${
              view === "account" ? "bg-blue-600 text-white" : "bg-gray-700 text-gray-300"
            }`}
          >
            Account Level
          </button>
          <button
            onClick={() => setView("model")}
            className={`px-3 py-1 text-sm rounded ${
              view === "model" ? "bg-blue-600 text-white" : "bg-gray-700 text-gray-300"
            }`}
          >
            Per Model
          </button>
        </div>

        <ResponsiveContainer width="100%" height={200}>
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#444" />
            <XAxis dataKey="date" stroke="#888" style={{ fontSize: "12px" }} />
            <YAxis stroke="#888" style={{ fontSize: "12px" }} />
            <Tooltip
              contentStyle={{ backgroundColor: "#1f2937", border: "1px solid #374151" }}
              labelStyle={{ color: "#fff" }}
            />
            <Legend wrapperStyle={{ fontSize: "12px" }} />
            <Line type="monotone" dataKey="cost" stroke="#10b981" name="Cost ($)" />
            <Line type="monotone" dataKey="input" stroke="#3b82f6" name="Input (M)" />
            <Line type="monotone" dataKey="output" stroke="#f59e0b" name="Output (M)" />
          </LineChart>
        </ResponsiveContainer>
      </div>
    );
  }

  // Model view
  if (!modelUsage || modelUsage.length === 0) {
    return <div className="text-center text-gray-400 py-4">No per-model data available</div>;
  }

  // Aggregate by model
  const modelMap = new Map<string, number>();
  modelUsage.forEach((usage: any) => {
    const current = modelMap.get(usage.model_name) || 0;
    modelMap.set(usage.model_name, current + usage.cost_usd);
  });

  const chartData = Array.from(modelMap.entries()).map(([model, cost]) => ({
    model,
    cost,
  }));

  return (
    <div className="space-y-3">
      <div className="flex gap-2">
        <button
          onClick={() => setView("account")}
          className={`px-3 py-1 text-sm rounded ${
            view === "account" ? "bg-blue-600 text-white" : "bg-gray-700 text-gray-300"
          }`}
        >
          Account Level
        </button>
        <button
          onClick={() => setView("model")}
          className={`px-3 py-1 text-sm rounded ${
            view === "model" ? "bg-blue-600 text-white" : "bg-gray-700 text-gray-300"
          }`}
        >
          Per Model
        </button>
      </div>

      <div className="space-y-2">
        {chartData.map(({ model, cost }) => (
          <div key={model} className="flex items-center justify-between bg-gray-700/30 rounded p-2">
            <span className="text-sm">{model}</span>
            <span className="text-sm font-semibold text-green-400">${cost.toFixed(4)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
