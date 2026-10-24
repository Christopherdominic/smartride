import type { NextConfig } from "next";

// Prevent publishing a build whose browser API calls silently target localhost.
if (process.env.VERCEL === "1") {
  const apiBase = process.env.NEXT_PUBLIC_API_BASE_URL;
  if (!apiBase) {
    throw new Error("Set NEXT_PUBLIC_API_BASE_URL to the public HTTPS backend URL before deploying.");
  }
  const url = new URL(apiBase);
  if (url.protocol !== "https:" || url.pathname !== "/" || url.search || url.hash || url.username || url.password) {
    throw new Error("NEXT_PUBLIC_API_BASE_URL must be an HTTPS origin without credentials, query, or path.");
  }
}

const config: NextConfig = { poweredByHeader: false };
export default config;
