"use client";
import { useEffect, useState } from "react";
import { api, type Config } from "@/lib/api";
export function ProviderBanner() {
  const [config, setConfig] = useState<Config>();
  const [unavailable, setUnavailable] = useState(false);
  useEffect(() => {
    let active = true;
    api<Config>("/demo")
      .then((c) => {
        if (active) setConfig(c);
      })
      .catch(() => {
        if (active) setUnavailable(true);
      });
    return () => {
      active = false;
    };
  }, []);
  return (
    <div className="demo-banner">
      <span className="dot" />
      {config?.provider === "mock"
        ? "Demo Postcode Data · Fictional offline fixtures · "
        : config?.provider === "nipost"
          ? `NIPOST ${config.environment === "sandbox" ? "sandbox" : "API"} · L1 postcode validity only · Addresses supplied by customers · `
          : unavailable
            ? "Postcode verification unavailable · "
            : ""}
      Shared demo workspace · Use sample recipient details only
    </div>
  );
}
