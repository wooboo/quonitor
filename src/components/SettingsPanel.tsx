import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X } from "lucide-react";

interface SettingsPanelProps {
  onClose: () => void;
}

export default function SettingsPanel({ onClose }: SettingsPanelProps) {
  const [refreshInterval, setRefreshInterval] = useState("300");
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const [threshold75, setThreshold75] = useState(true);
  const [threshold90, setThreshold90] = useState(true);
  const [threshold95, setThreshold95] = useState(true);
  const [dataRetention, setDataRetention] = useState("90");
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const [interval, notifs, t75, t90, t95, retention] = await Promise.all([
        invoke<string>("get_setting", { key: "refresh_interval_seconds" }),
        invoke<string>("get_setting", { key: "notifications_enabled" }),
        invoke<string>("get_setting", { key: "threshold_75_enabled" }),
        invoke<string>("get_setting", { key: "threshold_90_enabled" }),
        invoke<string>("get_setting", { key: "threshold_95_enabled" }),
        invoke<string>("get_setting", { key: "data_retention_days" }),
      ]);

      if (interval) setRefreshInterval(interval);
      if (notifs) setNotificationsEnabled(notifs === "true");
      if (t75) setThreshold75(t75 === "true");
      if (t90) setThreshold90(t90 === "true");
      if (t95) setThreshold95(t95 === "true");
      if (retention) setDataRetention(retention);
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);

    try {
      await Promise.all([
        invoke("set_setting", { key: "refresh_interval_seconds", value: refreshInterval }),
        invoke("set_setting", { key: "notifications_enabled", value: notificationsEnabled.toString() }),
        invoke("set_setting", { key: "threshold_75_enabled", value: threshold75.toString() }),
        invoke("set_setting", { key: "threshold_90_enabled", value: threshold90.toString() }),
        invoke("set_setting", { key: "threshold_95_enabled", value: threshold95.toString() }),
        invoke("set_setting", { key: "data_retention_days", value: dataRetention }),
      ]);

      onClose();
    } catch (error) {
      console.error("Failed to save settings:", error);
      alert("Failed to save settings");
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="bg-gray-800 rounded-lg border border-gray-700 p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold">Settings</h2>
        <button
          onClick={onClose}
          className="p-1 hover:bg-gray-700 rounded transition-colors"
        >
          <X className="w-5 h-5" />
        </button>
      </div>

      <div className="space-y-6">
        {/* Refresh Interval */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Refresh Interval (seconds)
          </label>
          <input
            type="number"
            value={refreshInterval}
            onChange={(e) => setRefreshInterval(e.target.value)}
            min="60"
            max="3600"
            className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
          />
          <p className="text-xs text-gray-400 mt-1">
            How often to check quota usage (minimum 60 seconds)
          </p>
        </div>

        {/* Notifications */}
        <div>
          <label className="flex items-center gap-2 text-sm font-medium text-gray-300 mb-3">
            <input
              type="checkbox"
              checked={notificationsEnabled}
              onChange={(e) => setNotificationsEnabled(e.target.checked)}
              className="w-4 h-4"
            />
            Enable Desktop Notifications
          </label>

          {notificationsEnabled && (
            <div className="ml-6 space-y-2">
              <label className="flex items-center gap-2 text-sm text-gray-400">
                <input
                  type="checkbox"
                  checked={threshold75}
                  onChange={(e) => setThreshold75(e.target.checked)}
                  className="w-4 h-4"
                />
                Notify at 75% usage
              </label>
              <label className="flex items-center gap-2 text-sm text-gray-400">
                <input
                  type="checkbox"
                  checked={threshold90}
                  onChange={(e) => setThreshold90(e.target.checked)}
                  className="w-4 h-4"
                />
                Notify at 90% usage
              </label>
              <label className="flex items-center gap-2 text-sm text-gray-400">
                <input
                  type="checkbox"
                  checked={threshold95}
                  onChange={(e) => setThreshold95(e.target.checked)}
                  className="w-4 h-4"
                />
                Notify at 95% usage (Critical)
              </label>
            </div>
          )}
        </div>

        {/* Data Retention */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Data Retention (days)
          </label>
          <input
            type="number"
            value={dataRetention}
            onChange={(e) => setDataRetention(e.target.value)}
            min="7"
            max="365"
            className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
          />
          <p className="text-xs text-gray-400 mt-1">
            Historical data older than this will be automatically deleted
          </p>
        </div>

        {/* Save Button */}
        <div className="flex gap-3 pt-4 border-t border-gray-700">
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 rounded transition-colors"
          >
            {isSaving ? "Saving..." : "Save Settings"}
          </button>
          <button
            onClick={onClose}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
