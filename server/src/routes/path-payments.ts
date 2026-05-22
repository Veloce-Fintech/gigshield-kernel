import { Router, Request, Response } from "express";
import {
  Keypair,
  TransactionBuilder,
  Operation,
  Asset,
  BASE_FEE,
  Networks,
  Horizon,
} from "stellar-sdk";
import { PathPaymentRequest, PathPaymentResult, PathResult } from "../types";

export function createPathPaymentRouter(): Router {
  const router = Router();
  const horizon = new Horizon.Server(
    process.env.HORIZON_URL || "https://horizon-testnet.stellar.org"
  );
  const networkPassphrase =
    process.env.STELLAR_NETWORK === "mainnet"
      ? Networks.PUBLIC
      : Networks.TESTNET;

  router.post("/find-paths", async (req: Request, res: Response) => {
    try {
      const { sourceAsset, sourceAmount, destinations } =
        req.body as PathPaymentRequest;

      if (!sourceAsset || !sourceAmount || !destinations?.length) {
        res.status(400).json({ error: "Missing required fields" });
        return;
      }

      const allResults: PathResult[] = [];

      for (const dest of destinations) {
        const source = new Asset(
          sourceAsset.split(":")[0],
          sourceAsset.split(":")[1] || ""
        );
        const destAsset = new Asset(
          dest.destinationAsset.split(":")[0],
          dest.destinationAsset.split(":")[1] || ""
        );

        const paths = await horizon
          .strictSendPaths(source, sourceAmount, [destAsset])
          .call();

        const bestPath = paths.records[0];
        if (bestPath) {
          const pathAssets = (bestPath.path || []).map(
            (p: { asset_type: string; asset_code: string; asset_issuer: string }) =>
              `${p.asset_code}:${p.asset_issuer || "native"}`
          );

          allResults.push({
            destinationAsset: dest.destinationAsset,
            destinationAmount: bestPath.destination_amount || "0",
            destinationAddress: dest.destinationAddress,
            sendMax: sourceAmount,
            path: pathAssets,
            fee: bestPath.destination_amount
              ? (
                  parseFloat(sourceAmount) - parseFloat(bestPath.destination_amount)
                ).toFixed(7)
              : "0",
          });
        }
      }

      const totalFee = allResults
        .reduce((sum, r) => sum + parseFloat(r.fee || "0"), 0)
        .toFixed(7);

      const result: PathPaymentResult = {
        sourceAsset,
        sourceAmount,
        paths: allResults,
        totalFee,
      };

      res.json(result);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : "Unknown error";
      res.status(500).json({ error: message });
    }
  });

  router.post("/execute", async (req: Request, res: Response) => {
    try {
      const {
        sourceAsset,
        sourceAmount,
        destinations,
      } = req.body as PathPaymentRequest;

      const signerSecret = process.env.SIGNER_SECRET_KEY;
      if (!signerSecret) {
        res.status(500).json({ error: "Signer key not configured" });
        return;
      }

      const sourceKeypair = Keypair.fromSecret(signerSecret);
      const sourceAccount = await horizon.loadAccount(sourceKeypair.publicKey());

      const source = new Asset(
        sourceAsset.split(":")[0],
        sourceAsset.split(":")[1] || ""
      );

      const txBuilder = new TransactionBuilder(sourceAccount, {
        fee: BASE_FEE,
        networkPassphrase,
      });

      for (const dest of destinations) {
        const destAsset = new Asset(
          dest.destinationAsset.split(":")[0],
          dest.destinationAsset.split(":")[1] || ""
        );

        txBuilder.addOperation(
          Operation.pathPaymentStrictSend({
            sendAsset: source,
            sendAmount: (
              parseFloat(sourceAmount) / destinations.length
            ).toString(),
            destination: dest.destinationAddress,
            destAsset,
            destMin: dest.destinationAmount,
          })
        );
      }

      const tx = txBuilder.setTimeout(30).build();
      tx.sign(sourceKeypair);

      const response = await horizon.submitTransaction(tx);

      res.json({
        success: true,
        hash: response.hash,
        ledger: response.ledger,
        envelope: response.envelope_xdr,
      });
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : "Unknown error";
      res.status(500).json({ error: message });
    }
  });

  return router;
}
