import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { QuotaData, AccountResponse, QuotaSnapshot, ModelUsage } from "../types";

export function useQuotaData() {
  const {
    data: quotas = [],
    isLoading: quotasLoading,
    refetch: refetchQuotas,
  } = useQuery<QuotaData[]>({
    queryKey: ["quotas"],
    queryFn: () => invoke<QuotaData[]>("get_all_quotas"),
    refetchInterval: 60000, // Refetch every minute
  });

  const {
    data: accounts = [],
    isLoading: accountsLoading,
    refetch: refetchAccounts,
  } = useQuery<AccountResponse[]>({
    queryKey: ["accounts"],
    queryFn: () => invoke<AccountResponse[]>("get_all_accounts"),
  });

  return {
    quotas,
    accounts,
    isLoading: quotasLoading || accountsLoading,
    refetch: () => {
      refetchQuotas();
      refetchAccounts();
    },
  };
}

export function useHistoricalData(accountId: string, days: number = 7) {
  return useQuery<QuotaSnapshot[]>({
    queryKey: ["historical", accountId, days],
    queryFn: () =>
      invoke<QuotaSnapshot[]>("get_historical_snapshots", { accountId, days }),
    enabled: !!accountId,
  });
}

export function useModelUsageHistory(accountId: string, days: number = 7) {
  return useQuery<ModelUsage[]>({
    queryKey: ["model-usage", accountId, days],
    queryFn: () =>
      invoke<ModelUsage[]>("get_model_usage_history", { accountId, days }),
    enabled: !!accountId,
  });
}
