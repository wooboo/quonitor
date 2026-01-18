export interface AccountResponse {
  id: string;
  provider: string;
  name: string;
  created_at: number;
  last_synced: number | null;
}

export interface QuotaData {
  account_id: string;
  timestamp: number;
  tokens_input: number | null;
  tokens_output: number | null;
  cost_usd: number | null;
  quota_limit: number | null;
  quota_remaining: number | null;
  model_breakdown: ModelData[];
  metadata: string | null;
}

export interface ModelData {
  model_name: string;
  tokens_input: number;
  tokens_output: number;
  cost_usd: number;
  request_count: number;
}

export interface QuotaSnapshot {
  id: number | null;
  account_id: string;
  timestamp: number;
  tokens_input: number | null;
  tokens_output: number | null;
  cost_usd: number | null;
  quota_limit: number | null;
  quota_remaining: number | null;
  metadata: string | null;
}

export interface ModelUsage {
  id: number | null;
  account_id: string;
  model_name: string;
  timestamp: number;
  tokens_input: number;
  tokens_output: number;
  cost_usd: number;
  request_count: number;
}

export interface Credentials {
  api_key?: string;
  oauth_token?: string;
  oauth_refresh_token?: string;
}
