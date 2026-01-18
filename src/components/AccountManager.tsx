import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { X, ExternalLink } from "lucide-react";
import type { Credentials } from "../types";

interface AccountManagerProps {
  onAccountAdded: () => void;
  onClose: () => void;
}

export default function AccountManager({ onAccountAdded, onClose }: AccountManagerProps) {
  const [provider, setProvider] = useState("openai");
  const [name, setName] = useState("");
  
  // Standard API Key
  const [apiKey, setApiKey] = useState("");
  
  // OAuth (Google)
  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const [authCode, setAuthCode] = useState("");
  const [authUrl, setAuthUrl] = useState("");
  
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState("");

  const handleGenerateAuthUrl = async () => {
    if (!clientId.trim() || !clientSecret.trim()) {
      setError("Client ID and Secret are required");
      return;
    }
    
    setIsSubmitting(true);
    try {
      // Use standard loopback redirect for installed apps
      const redirectUri = "urn:ietf:wg:oauth:2.0:oob"; 
      
      const [url, _csrf] = await invoke<[string, string]>("google_auth_start", {
        clientId,
        clientSecret,
        redirectUri,
      });
      
      setAuthUrl(url);
      await open(url);
      setError("");
    } catch (err) {
      console.error("Failed to generate URL:", err);
      setError("Failed to generate auth URL");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!name.trim()) {
      setError("Account name is required");
      return;
    }

    setIsSubmitting(true);

    try {
      let credentials: Credentials;

      if (provider === "google") {
        if (!authCode.trim()) {
          throw new Error("Authorization code is required");
        }
        
        // Exchange code for token
        credentials = await invoke<Credentials>("google_auth_finish", {
          clientId,
          clientSecret,
          redirectUri: "urn:ietf:wg:oauth:2.0:oob",
          code: authCode.trim(),
        });
      } else {
        // Standard API Key
        if (!apiKey.trim()) {
          throw new Error("API Key is required");
        }
        credentials = { api_key: apiKey };
      }

      await invoke("add_account", {
        request: {
          provider,
          name: name.trim(),
          credentials,
        },
      });

      onAccountAdded();
      // Reset form
      setName("");
      setApiKey("");
      setClientId("");
      setClientSecret("");
      setAuthCode("");
      setAuthUrl("");
    } catch (err) {
      console.error("Failed to add account:", err);
      setError(err instanceof Error ? err.message : "Failed to add account");
    } finally {
      setIsSubmitting(false);
    }
  };

  const renderGoogleAuth = () => (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">
          Client ID
        </label>
        <input
          type="text"
          value={clientId}
          onChange={(e) => setClientId(e.target.value)}
          className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-400 font-mono text-xs"
          placeholder="xxx.apps.googleusercontent.com"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">
          Client Secret
        </label>
        <input
          type="password"
          value={clientSecret}
          onChange={(e) => setClientSecret(e.target.value)}
          className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-400 font-mono text-xs"
        />
      </div>

      {!authUrl ? (
        <button
          type="button"
          onClick={handleGenerateAuthUrl}
          disabled={isSubmitting || !clientId || !clientSecret}
          className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 rounded transition-colors text-sm flex items-center justify-center gap-2"
        >
          <ExternalLink className="w-4 h-4" />
          Generate Auth URL
        </button>
      ) : (
        <div className="space-y-3 p-3 bg-gray-700/50 rounded border border-gray-600">
          <p className="text-xs text-gray-300">
            1. Authorize in the browser window that opened.
            <br />
            2. Copy the code provided by Google.
            <br />
            3. Paste it below:
          </p>
          <input
            type="text"
            value={authCode}
            onChange={(e) => setAuthCode(e.target.value)}
            placeholder="Paste 4/0... code here"
            className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white font-mono text-xs"
          />
        </div>
      )}
    </div>
  );

  return (
    <div className="bg-gray-800 rounded-lg border border-gray-700 p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold">Add New Account</h2>
        <button
          onClick={onClose}
          className="p-1 hover:bg-gray-700 rounded transition-colors"
        >
          <X className="w-5 h-5" />
        </button>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-900/30 border border-red-700 rounded text-red-300 text-sm">
          {error}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Provider
          </label>
          <select
            value={provider}
            onChange={(e) => {
              setProvider(e.target.value);
              setError("");
              setAuthUrl("");
            }}
            className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white"
          >
            <option value="openai">OpenAI</option>
            {/* Anthropic hidden until usage API is supported
            <option value="anthropic">Anthropic / Claude</option>
            */}
            <option value="google">Google / Antigravity</option>
            <option value="github">GitHub Copilot (Coming Soon)</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Account Name
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., Work Account, Personal"
            className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-400"
          />
        </div>

        {provider === "google" ? (
          renderGoogleAuth()
        ) : (
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              API Key
            </label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-400 font-mono"
            />
            <p className="text-xs text-gray-400 mt-1">
              Your API key is encrypted and stored securely in your system keychain
            </p>
          </div>
        )}

        <div className="flex gap-3 pt-2">
          <button
            type="submit"
            disabled={isSubmitting || (provider === "google" && !authCode) || (provider !== "google" && !apiKey)}
            className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 rounded transition-colors"
          >
            {isSubmitting ? "Adding..." : "Add Account"}
          </button>
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
