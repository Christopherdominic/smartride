export type Status =
  | "created"
  | "assigned"
  | "picked_up"
  | "out_for_delivery"
  | "delivered"
  | "failed";
export type Event = {
  id: string;
  event_type: string;
  created_at: string;
  metadata?: { seeded?: boolean };
};
export type Delivery = {
  id: string;
  merchant_id: string;
  recipient_name: string;
  recipient_phone: string;
  postcode: string;
  formatted_address: string;
  customer_address: string | null;
  landmark: string | null;
  address_source: "customer" | "offline_demo";
  postcode_provider: "mock" | "nipost";
  postcode_verified: boolean;
  postcode_environment: string | null;
  status: Status;
  rider_id: string | null;
  rider_name: string | null;
  parcel_description: string | null;
  delivery_instructions: string | null;
  created_at: string;
  events?: Event[];
  proof_confirmed: boolean;
};
export type Rider = {
  id: string;
  name: string;
  phone: string;
  active_deliveries: number;
};
export type Config = {
  mode: "demo";
  provider: "mock" | "nipost";
  environment: "offline" | "sandbox" | "production" | "unknown";
  customer_address_required: boolean;
  merchant: { id: string; name: string };
  postcodes: string[];
};
export type Address = {
  postcode: string;
  formatted_address: string | null;
  provider: "mock" | "nipost";
  provider_data: {
    verified: boolean;
    lookup_level?: number;
    environment?: string;
  };
};
export type Autocomplete = {
  segment: string;
  suggestions: { code: string; label: string }[];
};
export type Stats = {
  total: number;
  pending: number;
  active: number;
  completed: number;
  failed: number;
};
export async function api<T>(path: string, body?: unknown): Promise<T> {
  const response = await fetch(
    `${(process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080").replace(/\/$/, "")}/api/v1${path}`,
    {
      method: body === undefined ? "GET" : "POST",
      headers: { "Content-Type": "application/json" },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      cache: "no-store",
    },
  );
  const value = await response
    .json()
    .catch(() => ({ error: "API returned an unreadable response" }));
  if (!response.ok)
    throw new Error(value.error || `Request failed (${response.status})`);
  return value as T;
}
