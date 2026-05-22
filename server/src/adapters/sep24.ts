import axios from "axios";
import {
  AnchorConfig,
  Sep24DepositRequest,
  Sep24WithdrawRequest,
  Sep24Transaction,
} from "../types";

const ANCHORS: AnchorConfig[] = [
  {
    name: "YellowCard",
    apiUrl: process.env.ANCHOR_YELLOWCARD_API || "https://api.yellowcard.io",
    apiKey: process.env.ANCHOR_API_KEY || "",
    supportedRegions: ["NG", "GH", "KE", "ZA", "UG"],
    supportedAssets: ["USDC", "PYUSD"],
    feePercent: 0.01,
  },
  {
    name: "Anclap",
    apiUrl: process.env.ANCHOR_ANCLAP_API || "https://api.anclap.com",
    apiKey: process.env.ANCHOR_API_KEY || "",
    supportedRegions: ["AR", "BR", "CL", "CO", "MX"],
    supportedAssets: ["USDC", "PYUSD"],
    feePercent: 0.015,
  },
];

export class Sep24Adapter {
  private anchors: Map<string, AnchorConfig>;

  constructor() {
    this.anchors = new Map(ANCHORS.map((a) => [a.name.toLowerCase(), a]));
  }

  listAnchors(region?: string): AnchorConfig[] {
    if (region) {
      return ANCHORS.filter((a) =>
        a.supportedRegions.includes(region.toUpperCase())
      );
    }
    return ANCHORS;
  }

  async deposit(req: Sep24DepositRequest): Promise<Sep24Transaction> {
    const anchor = this.anchors.get(req.anchor.toLowerCase());
    if (!anchor) throw new Error(`Unknown anchor: ${req.anchor}`);

    const response = await axios.post(
      `${anchor.apiUrl}/sep24/deposit`,
      {
        asset_code: req.asset_code,
        amount: req.amount,
        account: req.destination,
        memo: req.memo,
        memo_type: req.memo_type,
      },
      {
        headers: {
          Authorization: `Bearer ${anchor.apiKey}`,
          "Content-Type": "application/json",
        },
      }
    );

    return {
      id: response.data.id,
      kind: "deposit",
      status: "pending",
      amount_in: req.amount,
      amount_out: "0",
      amount_fee: "0",
      started_at: new Date().toISOString(),
      stellar_transaction_id: response.data.stellar_transaction_id,
    };
  }

  async withdraw(req: Sep24WithdrawRequest): Promise<Sep24Transaction> {
    const anchor = this.anchors.get(req.anchor.toLowerCase());
    if (!anchor) throw new Error(`Unknown anchor: ${req.anchor}`);

    const response = await axios.post(
      `${anchor.apiUrl}/sep24/withdraw`,
      {
        asset_code: req.asset_code,
        amount: req.amount,
        account: req.source,
        bank_account: req.bank_account,
      },
      {
        headers: {
          Authorization: `Bearer ${anchor.apiKey}`,
          "Content-Type": "application/json",
        },
      }
    );

    return {
      id: response.data.id,
      kind: "withdrawal",
      status: "pending",
      amount_in: req.amount,
      amount_out: "0",
      amount_fee: "0",
      started_at: new Date().toISOString(),
      external_transaction_id: response.data.external_transaction_id,
    };
  }

  async getTransactionStatus(txId: string): Promise<Sep24Transaction> {
    for (const anchor of ANCHORS) {
      try {
        const response = await axios.get(
          `${anchor.apiUrl}/sep24/transaction/${txId}`,
          {
            headers: {
              Authorization: `Bearer ${anchor.apiKey}`,
            },
          }
        );
        return {
          id: response.data.id,
          kind: response.data.kind,
          status: response.data.status,
          amount_in: response.data.amount_in,
          amount_out: response.data.amount_out,
          amount_fee: response.data.amount_fee,
          started_at: response.data.started_at,
          completed_at: response.data.completed_at,
          stellar_transaction_id: response.data.stellar_transaction_id,
          external_transaction_id: response.data.external_transaction_id,
        };
      } catch {
        continue;
      }
    }
    throw new Error(`Transaction ${txId} not found on any anchor`);
  }
}
