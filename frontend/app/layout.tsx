import type { Metadata } from "next";
import Link from "next/link";
import { Package, ArrowUpRight } from "lucide-react";
import "./globals.css";
import { ProviderBanner } from "@/components/provider-banner";
export const metadata: Metadata = {
  title: "SmartRide — One postcode. Every delivery.",
  description:
    "Postcode-powered delivery coordination for Nigeria. NIPOST L1 postcode verification with customer-provided delivery addresses.",
};
export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <header>
          <Link href="/" className="brand">
            <span className="brand-icon">
              <Package size={23} />
            </span>
            SmartRide<span className="brand-dot">.</span>
          </Link>
          <nav aria-label="Main navigation">
            <Link href="/dashboard">Overview</Link>
            <Link href="/rider">Rider workspace</Link>
            <Link href="/demo" className="nav-cta">
              Launch demo <ArrowUpRight size={16} />
            </Link>
          </nav>
        </header>
        <ProviderBanner />
        {children}
        <footer>
          <Link href="/" className="brand">
            SmartRide.
          </Link>
          <span>One postcode. Every delivery.</span>
          <span>Built for Nigeria’s last mile.</span>
        </footer>
      </body>
    </html>
  );
}
