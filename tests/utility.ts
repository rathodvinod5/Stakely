import { BN } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";

/**
 * Formats a number, BN, or BigInt with underscore separators.
 * Supports both whole lamports and decimal SOL.
 */
export const formatWithUnderscores = (
  value: number | string | BN | bigint,
): string => {
  const strValue = value.toString();

  // Split to handle decimals if they exist (e.g., 1250.55 SOL)
  const parts = strValue.split(".");

  // Regex to add underscores every 3 digits
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, "_");

  return parts.join(".");
};

/**
 * Converts lamports to SOL and formats with underscores.
 */
export const formatLamportsToSol = (lamports: BN | bigint | number): string => {
  const sol = Number(lamports) / LAMPORTS_PER_SOL;
  return formatWithUnderscores(sol);
};
