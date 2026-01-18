import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Trash2, ChevronDown, ChevronUp, RefreshCw } from "lucide-react";
import TrendChart from "./TrendChart";
import type { QuotaData, AccountResponse } from "../types";

interface QuotaCardProps {
  quota: QuotaData;
  account?: AccountResponse;
  onDelete: () => void;
}

export default function QuotaCard({ quota, account, onDelete }: QuotaCardProps) {
  const [showModels, setShowModels] = useState(false);
  const [showChart, setShowChart] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const handleDelete = async () => {
    if (!confirm(`Delete account "${account?.name || quota.account_id}"?`)) {
      return;
    }

    setIsDeleting(true);
    try {
      await invoke("remove_account", { accountId: quota.account_id });
      onDelete();
    } catch (error) {
      console.error("Failed to delete account:", error);
      alert("Failed to delete account");
    } finally {
      setIsDeleting(false);
    }
  };

  const formatNumber = (num: number | null) => {
    if (num === null) return "N/A";
    if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(2)}K`;
    return num.toString();
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const getProviderColor = (provider: string) => {
    switch (provider) {
      case "openai":
        return "bg-green-600";
      case "anthropic":
        return "bg-purple-600";
      case "google":
        return "bg-blue-600";
      case "github":
        return "bg-gray-600";
      default:
        return "bg-gray-600";
    }
  };

  const totalTokens = (quota.tokens_input || 0) + (quota.tokens_output || 0);
  const hasModelBreakdown = quota.model_breakdown && quota.model_breakdown.length > 0;

  return (
    <div className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
      {/* Header */}
      <div className={`${getProviderColor(account?.provider || "")} px-4 py-3 flex items-center justify-between`}>
        <div>
          <h3 className="font-semibold text-white">{account?.name || quota.account_id}</h3>
          <p className="text-xs text-white/80 uppercase">{account?.provider || "Unknown"}</p>
        </div>
        <button
          onClick={handleDelete}
          disabled={isDeleting}
          className="p-2 hover:bg-white/10 rounded transition-colors disabled:opacity-50"
        >
          <Trash2 className="w-4 h-4 text-white" />
        </button>
      </div>

      {/* Content */}
      <div className="p-4 space-y-4">
        {/* Token Usage */}
        <div className="grid grid-cols-2 gap-3">
          <div>
            <p className="text-xs text-gray-400 mb-1">Input Tokens</p>
            <p className="text-lg font-semibold">{formatNumber(quota.tokens_input)}</p>
          </div>
          <div>
            <p className="text-xs text-gray-400 mb-1">Output Tokens</p>
            <p className="text-lg font-semibold">{formatNumber(quota.tokens_output)}</p>
          </div>
        </div>

        {/* Cost */}
        <div>
          <p className="text-xs text-gray-400 mb-1">Estimated Cost</p>
          <p className="text-2xl font-bold text-green-400">
            ${quota.cost_usd?.toFixed(2) || "0.00"}
          </p>
        </div>

        {/* Progress Bar (if limits available) */}
        {quota.quota_limit && quota.quota_remaining !== null && (
          <div>
            <div className="flex justify-between text-xs text-gray-400 mb-1">
              <span>Usage</span>
              <span>
                {formatNumber(quota.quota_limit - quota.quota_remaining)} / {formatNumber(quota.quota_limit)}
              </span>
            </div>
            <div className="w-full bg-gray-700 rounded-full h-2">
              <div
                className="bg-blue-500 h-2 rounded-full transition-all"
                style={{
                  width: `${Math.min(
                    ((quota.quota_limit - quota.quota_remaining) / quota.quota_limit) * 100,
                    100
                  )}%`,
                }}
              />
            </div>
          </div>
        )}

        {/* Model Breakdown */}
        {hasModelBreakdown && (
          <div>
            <button
              onClick={() => setShowModels(!showModels)}
              className="flex items-center justify-between w-full text-sm text-gray-300 hover:text-white transition-colors"
            >
              <span>Model Breakdown ({quota.model_breakdown.length})</span>
              {showModels ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
            </button>

            {showModels && (
              <div className="mt-3 space-y-2">
                {quota.model_breakdown.map((model) => (
                  <div key={model.model_name} className="bg-gray-700/50 rounded p-3">
                    <div className="flex justify-between items-start mb-2">
                      <span className="font-medium text-sm">{model.model_name}</span>
                      <span className="text-sm text-green-400">${model.cost_usd.toFixed(4)}</span>
                    </div>
                    <div className="grid grid-cols-3 gap-2 text-xs text-gray-400">
                      <div>
                        <span className="block">Input</span>
                        <span className="text-white">{formatNumber(model.tokens_input)}</span>
                      </div>
                      <div>
                        <span className="block">Output</span>
                        <span className="text-white">{formatNumber(model.tokens_output)}</span>
                      </div>
                      <div>
                        <span className="block">Requests</span>
                        <span className="text-white">{model.request_count}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Chart Toggle */}
        <div>
          <button
            onClick={() => setShowChart(!showChart)}
            className="flex items-center justify-between w-full text-sm text-gray-300 hover:text-white transition-colors"
          >
            <span>Historical Trends</span>
            {showChart ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          </button>

          {showChart && (
            <div className="mt-3">
              <TrendChart accountId={quota.account_id} />
            </div>
          )}
        </div>

        {/* Last Updated */}
        <div className="text-xs text-gray-500 border-t border-gray-700 pt-3">
          Last updated: {formatDate(quota.timestamp)}
        </div>
      </div>
    </div>
  );
}
