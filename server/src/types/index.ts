export interface AnchorConfig {
  name: string;
  apiUrl: string;
  apiKey: string;
  supportedRegions: string[];
  supportedAssets: string[];
  feePercent: number;
}

export interface Sep24DepositRequest {
  asset_code: string;
  amount: string;
  anchor: string;
  destination: string;
  memo?: string;
  memo_type?: string;
}

export interface Sep24WithdrawRequest {
  asset_code: string;
  amount: string;
  anchor: string;
  source: string;
  bank_account?: string;
}

export interface Sep24Transaction {
  id: string;
  kind: "deposit" | "withdrawal";
  status: "pending" | "completed" | "error";
  amount_in: string;
  amount_out: string;
  amount_fee: string;
  started_at: string;
  completed_at?: string;
  stellar_transaction_id?: string;
  external_transaction_id?: string;
}

export interface PathPaymentRequest {
  sourceAsset: string;
  sourceAmount: string;
  destinations: PathDestination[];
}

export interface PathDestination {
  destinationAsset: string;
  destinationAmount: string;
  destinationAddress: string;
}

export interface PathPaymentResult {
  sourceAsset: string;
  sourceAmount: string;
  paths: PathResult[];
  totalFee: string;
}

export interface PathResult {
  destinationAsset: string;
  destinationAmount: string;
  destinationAddress: string;
  sendMax: string;
  path: string[];
  fee: string;
}
