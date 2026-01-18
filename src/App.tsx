import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Settings, Plus } from "lucide-react";
import QuotaCard from "./components/QuotaCard";
import AccountManager from "./components/AccountManager";
import SettingsPanel from "./components/SettingsPanel";
import { useQuotaData } from "./hooks/useQuotaData";

function App() {
  const [showAccountManager, setShowAccountManager] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const { quotas, accounts, refetch, isLoading } = useQuotaData();

  useEffect(() => {
    const unlisten = listen("refresh-requested", () => {
      handleRefresh();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await invoke("refresh_now");
      // Wait a moment for the refresh to complete
      setTimeout(() => {
        refetch();
        setIsRefreshing(false);
      }, 1000);
    } catch (error) {
      console.error("Failed to refresh:", error);
      setIsRefreshing(false);
    }
  };

  const handleAccountAdded = () => {
    setShowAccountManager(false);
    refetch();
  };

  const handleAccountDeleted = () => {
    refetch();
  };

  const getOverallStatus = () => {
    if (quotas.length === 0) return "No accounts configured";

    const totalCost = quotas.reduce((sum, q) => sum + (q.cost_usd || 0), 0);
    const totalInput = quotas.reduce((sum, q) => sum + (q.tokens_input || 0), 0);
    const totalOutput = quotas.reduce((sum, q) => sum + (q.tokens_output || 0), 0);

    return {
      accountCount: quotas.length,
      totalCost: totalCost.toFixed(2),
      totalInput: (totalInput / 1000000).toFixed(2),
      totalOutput: (totalOutput / 1000000).toFixed(2),
    };
  };

  const status = getOverallStatus();

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">Quonitor</h1>
            <p className="text-sm text-gray-400 mt-1">
              {typeof status === "string" ? (
                status
              ) : (
                <>
                  {status.accountCount} {status.accountCount === 1 ? "account" : "accounts"} •
                  ${status.totalCost} total • {status.totalInput}M input / {status.totalOutput}M output tokens
                </>
              )}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={handleRefresh}
              disabled={isRefreshing}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 rounded-lg transition-colors"
            >
              <RefreshCw className={`w-4 h-4 ${isRefreshing ? "animate-spin" : ""}`} />
              Refresh
            </button>
            <button
              onClick={() => setShowSettings(!showSettings)}
              className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
            >
              <Settings className="w-5 h-5" />
            </button>
            <button
              onClick={() => setShowAccountManager(!showAccountManager)}
              className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition-colors"
            >
              <Plus className="w-4 h-4" />
              Add Account
            </button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="p-6">
        {showAccountManager && (
          <div className="mb-6">
            <AccountManager onAccountAdded={handleAccountAdded} onClose={() => setShowAccountManager(false)} />
          </div>
        )}

        {showSettings && (
          <div className="mb-6">
            <SettingsPanel onClose={() => setShowSettings(false)} />
          </div>
        )}

        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-gray-400">Loading...</div>
          </div>
        ) : quotas.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-center">
            <p className="text-gray-400 text-lg mb-4">No accounts configured yet</p>
            <button
              onClick={() => setShowAccountManager(true)}
              className="flex items-center gap-2 px-6 py-3 bg-green-600 hover:bg-green-700 rounded-lg transition-colors"
            >
              <Plus className="w-5 h-5" />
              Add Your First Account
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
            {quotas.map((quota) => {
              const account = accounts.find((a) => a.id === quota.account_id);
              return (
                <QuotaCard
                  key={quota.account_id}
                  quota={quota}
                  account={account}
                  onDelete={handleAccountDeleted}
                />
              );
            })}
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
