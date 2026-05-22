import express from "express";
import cors from "cors";
import dotenv from "dotenv";
import { createPathPaymentRouter } from "./routes/path-payments";
import { Sep24Adapter } from "./adapters/sep24";

dotenv.config();

const PORT = process.env.PORT || 4000;
const sep24 = new Sep24Adapter();

const app = express();

app.use(cors());
app.use(express.json());

app.get("/health", (req, res) => {
  res.json({ status: "ok", service: "gigshield-kernel" });
});

app.use("/api/v1/path-payments", createPathPaymentRouter());

app.get("/api/v1/anchors", (req, res) => {
  const region = req.query.region as string | undefined;
  res.json({ anchors: sep24.listAnchors(region) });
});

app.post("/api/v1/anchors/deposit", async (req, res) => {
  try {
    const result = await sep24.deposit(req.body);
    res.json(result);
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : "Unknown error";
    res.status(400).json({ error: message });
  }
});

app.post("/api/v1/anchors/withdraw", async (req, res) => {
  try {
    const result = await sep24.withdraw(req.body);
    res.json(result);
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : "Unknown error";
    res.status(400).json({ error: message });
  }
});

app.get("/api/v1/anchors/transaction/:txId", async (req, res) => {
  try {
    const result = await sep24.getTransactionStatus(req.params.txId);
    res.json(result);
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : "Unknown error";
    res.status(404).json({ error: message });
  }
});

app.listen(PORT, () => {
  console.log(`GigShield Kernel running on port ${PORT}`);
});
