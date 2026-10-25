"use client";
import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import {
  ArrowRight,
  Plus,
  Package,
  CheckCircle2,
  MapPin,
  Truck,
  RefreshCw,
} from "lucide-react";
import {
  api,
  type Config,
  type Delivery,
  type Rider,
  type Address,
  type Stats,
  type Autocomplete,
} from "@/lib/api";
const label = (s: string) => s.replaceAll("_", " ");
export function Badge({ status }: { status: string }) {
  return <span className={`badge ${status}`}>{label(status)}</span>;
}
function ErrorBox({ message }: { message: string }) {
  return message ? (
    <div role="alert" className="error">
      {message}
    </div>
  ) : null;
}
function useLoad<T>(load: () => Promise<T>) {
  const [data, setData] = useState<T>();
  const [error, setError] = useState("");
  const refresh = useCallback(async () => {
    setError("");
    try {
      setData(await load());
    } catch (e) {
      setError((e as Error).message);
    }
  }, [load]);
  useEffect(() => {
    void refresh();
  }, [refresh]);
  return { data, error, refresh };
}
const loadDashboard = () =>
  Promise.all([api<Delivery[]>("/deliveries"), api<Stats>("/stats")]);
export function Dashboard() {
  const { data, error, refresh } = useLoad(loadDashboard);
  return (
    <main className="workspace">
      <div className="page-heading">
        <div>
          <div className="eyebrow">MERCHANT WORKSPACE</div>
          <h1>
            Delivery overview<span className="brand-dot">.</span>
          </h1>
          <p>A little clarity for every mile ahead.</p>
        </div>
        <Link href="/dashboard/deliveries/new" className="button">
          <Plus size={18} />
          Create delivery
        </Link>
      </div>
      <ErrorBox message={error} />
      {!data && !error ? (
        <p role="status">Loading your workspace…</p>
      ) : (
        data && (
          <>
            <div className="stat-grid">
              {[
                ["Total deliveries", data[1].total, Package],
                ["Pending dispatch", data[1].pending, MapPin],
                ["On the move", data[1].active, Truck],
                ["Delivered", data[1].completed, CheckCircle2],
              ].map(([name, n, Icon]) => {
                const I = Icon as typeof Package;
                return (
                  <article className="stat" key={String(name)}>
                    <span>
                      {String(name)}
                      <I size={19} />
                    </span>
                    <strong>{String(n)}</strong>
                    <small>Dedicated demo merchant</small>
                  </article>
                );
              })}
            </div>
            <section className="panel">
              <div className="panel-heading">
                <div>
                  <h2>Recent deliveries</h2>
                  <p>Every parcel. Every handoff. All in one place.</p>
                </div>
                <button
                  className="icon-button"
                  onClick={() => void refresh()}
                  aria-label="Refresh deliveries"
                >
                  <RefreshCw size={18} />
                </button>
              </div>
              <DeliveryTable deliveries={data[0]} />
            </section>
            <section className="panel breakdown">
              <h2>Delivery status breakdown</h2>
              {Object.entries({
                created: data[1].pending,
                active: data[1].active,
                delivered: data[1].completed,
                failed: data[1].failed,
              }).map(([s, n]) => (
                <div key={s}>
                  <span>{label(s)}</span>
                  <progress max={Math.max(1, data[1].total)} value={n} />
                  <strong>{n}</strong>
                </div>
              ))}
            </section>
          </>
        )
      )}
    </main>
  );
}
function DeliveryTable({ deliveries }: { deliveries: Delivery[] }) {
  return !deliveries.length ? (
    <div className="empty">
      <Package />
      <h3>Your next delivery starts here.</h3>
      <Link href="/dashboard/deliveries/new">Create your first delivery →</Link>
    </div>
  ) : (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Delivery / recipient</th>
            <th>Destination</th>
            <th>Rider</th>
            <th>Status</th>
            <th>
              <span className="sr-only">Details</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {deliveries.map((d) => (
            <tr key={d.id}>
              <td>
                <Link
                  className="delivery-ref"
                  href={`/dashboard/deliveries/${d.id}`}
                >
                  SR-{d.id.slice(0, 8).toUpperCase()}
                </Link>
                <small>{d.recipient_name}</small>
              </td>
              <td>
                <strong>{d.postcode}</strong>
                <small>{d.formatted_address}</small>
              </td>
              <td>{d.rider_name || "Awaiting assignment"}</td>
              <td>
                <Badge status={d.status} />
              </td>
              <td>
                <Link
                  aria-label={`View delivery ${d.id}`}
                  href={`/dashboard/deliveries/${d.id}`}
                >
                  <ArrowRight size={18} />
                </Link>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
const loadConfig = () => api<Config>("/demo");
export function CreateForm({ guided = false }: { guided?: boolean }) {
  const { data: config, error: loadError } = useLoad(loadConfig);
  const [postcode, setPostcode] = useState("EK01A03FK01");
  const [address, setAddress] = useState<Address>();
  const [suggestions, setSuggestions] = useState<Autocomplete>();
  const [searchError, setSearchError] = useState("");
  const [searchBusy, setSearchBusy] = useState(false);
  const [customerAddress, setCustomerAddress] = useState("");
  const live = config?.provider === "nipost";
  useEffect(() => {
    if (config?.provider === "nipost") setPostcode("");
  }, [config]);
  async function search() {
    setSearchBusy(true);
    setSearchError("");
    setSuggestions(undefined);
    try {
      setSuggestions(
        await api<Autocomplete>(
          `/addresses/autocomplete?q=${encodeURIComponent(postcode)}`,
        ),
      );
    } catch (e) {
      setSearchError((e as Error).message);
    } finally {
      setSearchBusy(false);
    }
  }
  function selectSuggestion(code: string) {
    const compact = postcode.toUpperCase().replace(/[\s-]/g, "");
    const index = { state: 0, lga: 1, district: 2, area: 3, unit: 4 }[
      suggestions?.segment || ""
    ];
    const widths = [2, 2, 3, 2, 2];
    let next = code;
    if (
      index !== undefined &&
      !code.includes("-") &&
      code.length <= widths[index]
    ) {
      let offset = 0;
      const prefix = widths.slice(0, index).map((width) => {
        const part = compact.slice(offset, offset + width);
        offset += width;
        return part;
      });
      if (prefix.every((part, i) => part.length === widths[i]))
        next = [...prefix, code].join("-") + (index < 4 ? "-" : "");
    }
    setPostcode(next);
    setAddress(undefined);
    setSuggestions(undefined);
  }
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const router = useRouter();
  async function resolve() {
    setBusy(true);
    setError("");
    setAddress(undefined);
    try {
      setAddress(
        await api<Address>(
          `/addresses/resolve?postcode=${encodeURIComponent(postcode)}`,
        ),
      );
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!config || !address || (live && !address.provider_data.verified))
      return;
    const f = new FormData(e.currentTarget);
    setBusy(true);
    setError("");
    try {
      const d = await api<Delivery>("/deliveries", {
        merchant_id: config.merchant.id,
        recipient_name: f.get("name"),
        recipient_phone: f.get("phone"),
        destination_postcode: address.postcode,
        customer_address: customerAddress.trim() || undefined,
        landmark: f.get("landmark"),
        parcel_description: f.get("parcel"),
        delivery_instructions: f.get("instructions"),
      });
      router.push(`/dashboard/deliveries/${d.id}${guided ? "?guided=1" : ""}`);
    } catch (e) {
      setError((e as Error).message);
      setBusy(false);
    }
  }
  return (
    <main className="workspace narrow">
      <Link href="/dashboard" className="back">
        ← Overview
      </Link>
      <div className="eyebrow">
        {guided ? "YOUR THREE-MINUTE WALKTHROUGH" : "NEW DELIVERY"}
      </div>
      <h1>
        {guided ? "A postcode. A parcel. Let’s go." : "Send something good."}
      </h1>
      <p>
        {guided
          ? "Verify a postcode, supply the delivery address, then follow assignment through to OTP confirmation."
          : "Start with a postcode. We’ll connect the rest."}
      </p>
      <ErrorBox message={error || loadError} />
      {config && (
        <form onSubmit={submit} className="panel form">
          <label>
            Demo merchant
            <select aria-label="Demo merchant">
              <option>{config.merchant.name}</option>
            </select>
          </label>
          <div className="form-divider">
            <span>01</span>
            <h2>Verify the destination postcode</h2>
          </div>
          <label>
            Digital postcode
            <div className="input-action">
              <input
                value={postcode}
                disabled={busy || searchBusy}
                maxLength={30}
                placeholder={live ? "AA-99-H77-BB-55" : "EK-01-A03-FK-01"}
                onChange={(e) => {
                  setPostcode(e.target.value);
                  setAddress(undefined);
                  setSuggestions(undefined);
                }}
                required
              />
              <button
                type="button"
                onClick={() => void resolve()}
                disabled={busy}
              >
                {live ? "Verify postcode" : "Resolve postcode"}
              </button>
            </div>
          </label>
          {!live && (
            <p className="hint">
              Try{" "}
              {config.postcodes.map((p) => (
                <button
                  key={p}
                  type="button"
                  className="postcode-option"
                  onClick={() => {
                    setPostcode(p);
                    setAddress(undefined);
                  }}
                >
                  {p}
                </button>
              ))}
            </p>
          )}
          {live && (
            <>
              <p className="hint">
                Use a postcode from your key’s{" "}
                {config.environment === "sandbox"
                  ? "sandbox dataset"
                  : "configured dataset"}
                . Lookup L1 checks validity only.
              </p>
              <button
                type="button"
                className="button secondary"
                disabled={busy || searchBusy || postcode.trim().length < 2}
                onClick={() => void search()}
              >
                {searchBusy ? "Searching…" : "Find postcode suggestions"}
              </button>
              <ErrorBox message={searchError} />
              {suggestions && (
                <div aria-live="polite">
                  <p className="hint">
                    {suggestions.suggestions.length
                      ? `Suggestions for ${suggestions.segment}; select one, then verify the complete postcode.`
                      : "No suggestions returned. You can still enter a complete postcode and verify it."}
                  </p>
                  {suggestions.suggestions.map((s) => (
                    <button
                      type="button"
                      className="postcode-option"
                      key={s.code}
                      onClick={() => selectSuggestion(s.code)}
                    >
                      {s.label} ({s.code})
                    </button>
                  ))}
                </div>
              )}
            </>
          )}
          {address && (
            <div className="address-preview">
              <MapPin />
              <div>
                <strong>
                  {address.provider === "nipost" &&
                  address.provider_data.verified
                    ? "Postcode verified by NIPOST"
                    : address.formatted_address}
                </strong>
                <small>
                  {address.postcode} ·{" "}
                  {address.provider === "nipost"
                    ? `${address.provider_data.environment === "sandbox" ? "Sandbox · " : ""}L1 validity only; street address is not verified`
                    : "Fictional demonstration address"}
                </small>
              </div>
              <CheckCircle2 />
            </div>
          )}
          <label>
            Full delivery address{" "}
            {live ? "(required)" : "(optional for offline demo)"}
            <textarea
              name="customer_address"
              value={customerAddress}
              onChange={(e) => setCustomerAddress(e.target.value)}
              required={live}
              minLength={5}
              maxLength={1000}
              placeholder="Customer-provided house number, street, locality and city"
            />
            <span className="hint">
              Customer-provided address; NIPOST L1 does not verify street
              addresses or coordinates.
            </span>
          </label>
          <label>
            Landmark or delivery directions
            <input
              name="landmark"
              maxLength={500}
              placeholder="e.g. Opposite the community centre"
            />
          </label>
          <div className="form-divider">
            <span>02</span>
            <h2>Add parcel details</h2>
          </div>
          <div className="form-grid">
            <label>
              Recipient name
              <input
                name="name"
                required
                maxLength={120}
                placeholder="Demo Recipient"
                defaultValue={guided ? "Demo Recipient" : ""}
              />
            </label>
            <label>
              Recipient phone
              <input
                name="phone"
                required
                type="tel"
                maxLength={30}
                placeholder="+2340000000000"
                defaultValue={guided ? "+2340000000000" : ""}
              />
            </label>
          </div>
          <label>
            Parcel description
            <input
              name="parcel"
              maxLength={1000}
              placeholder="e.g. A box of handmade goods"
            />
          </label>
          <label>
            Delivery instructions
            <textarea
              name="instructions"
              maxLength={1000}
              placeholder="Anything your rider should know?"
            />
          </label>
          <div className="form-bottom">
            <span>Sample details only. This workspace is public.</span>
            <button
              className="button"
              disabled={
                busy || !address || (live && customerAddress.trim().length < 5)
              }
            >
              {busy ? "Working…" : "Create delivery"}
              <ArrowRight size={18} />
            </button>
          </div>
        </form>
      )}
    </main>
  );
}
const loadRiders = () => api<Rider[]>("/riders");
export function Detail({ id }: { id: string }) {
  const load = useCallback(() => api<Delivery>(`/deliveries/${id}`), [id]);
  const { data: d, error, refresh } = useLoad(load);
  const { data: riders, error: riderError } = useLoad(loadRiders);
  return (
    <main className="workspace">
      <Link className="back" href="/dashboard">
        ← All deliveries
      </Link>
      <ErrorBox message={error} />
      {!d && !error ? (
        <p role="status">Loading delivery…</p>
      ) : (
        d && (
          <>
            <div className="page-heading">
              <div>
                <div className="eyebrow">DELIVERY DETAILS</div>
                <h1>SR-{d.id.slice(0, 8).toUpperCase()}</h1>
                <p>Created {new Date(d.created_at).toLocaleString()}</p>
              </div>
              <Badge status={d.status} />
            </div>
            <div className="detail-grid">
              <div>
                <section className="panel detail-card">
                  <h2>Destination & recipient</h2>
                  <div className="address-preview">
                    <MapPin />
                    <div>
                      <strong>{d.postcode}</strong>
                      <p>{d.formatted_address}</p>
                      <small>
                        {d.address_source === "customer"
                          ? "Customer-provided address · not verified by NIPOST"
                          : "Historical offline demo address"}
                      </small>
                      {d.postcode_verified && (
                        <small>
                          Postcode verified by NIPOST · L1 validity only
                          {d.postcode_environment === "sandbox"
                            ? " · Sandbox"
                            : ""}
                        </small>
                      )}
                    </div>
                  </div>
                  <dl>
                    <dt>Landmark</dt>
                    <dd>{d.landmark || "No landmark supplied"}</dd>
                    <dt>Recipient</dt>
                    <dd>{d.recipient_name}</dd>
                    <dt>Phone</dt>
                    <dd>{d.recipient_phone}</dd>
                    <dt>Parcel</dt>
                    <dd>{d.parcel_description || "No description added"}</dd>
                    <dt>Instructions</dt>
                    <dd>
                      {d.delivery_instructions || "No additional instructions"}
                    </dd>
                    <dt>Assigned rider</dt>
                    <dd>{d.rider_name || "Awaiting assignment"}</dd>
                    <dt>Proof of delivery</dt>
                    <dd>
                      {d.proof_confirmed
                        ? "Confirmed with OTP"
                        : d.status === "delivered"
                          ? "Seeded example — no OTP confirmation"
                          : "Awaiting OTP confirmation"}
                    </dd>
                  </dl>
                </section>
                <section className="panel detail-card">
                  <h2>Next step</h2>
                  <ErrorBox message={riderError} />
                  <DeliveryActions
                    delivery={d}
                    riders={riders || []}
                    onChange={refresh}
                  />
                </section>
              </div>
              <section className="panel detail-card">
                <h2>Delivery timeline</h2>
                <p>Every handoff, from dispatch to doorstep.</p>
                <ol className="timeline">
                  {d.events?.map((e) => (
                    <li key={e.id}>
                      <span className="timeline-dot">
                        <CheckCircle2 size={15} />
                      </span>
                      <strong>{label(e.event_type)}</strong>
                      <small>
                        {new Date(e.created_at).toLocaleString()}
                        {e.metadata?.seeded ? " · Seeded example" : ""}
                      </small>
                    </li>
                  ))}
                </ol>
              </section>
            </div>
          </>
        )
      )}
    </main>
  );
}
function DeliveryActions({
  delivery: d,
  riders,
  onChange,
}: {
  delivery: Delivery;
  riders: Rider[];
  onChange: () => Promise<void>;
}) {
  const [rider, setRider] = useState("");
  const [code, setCode] = useState("");
  const [demoOtp, setDemoOtp] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  async function act(action: string, body: unknown) {
    setError("");
    setBusy(true);
    try {
      const result = await api<{ demo_otp?: string }>(
        `/deliveries/${d.id}/${action}`,
        body,
      );
      if (result.demo_otp) setDemoOtp(result.demo_otp);
      await onChange();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  return (
    <div className="delivery-actions">
      <ErrorBox message={error} />
      {d.status === "created" ? (
        <>
          <label>
            Assign a demo rider
            <select value={rider} onChange={(e) => setRider(e.target.value)}>
              <option value="">Choose a rider</option>
              {riders.map((r) => (
                <option value={r.id} key={r.id}>
                  {r.name} · {r.active_deliveries} active
                </option>
              ))}
            </select>
          </label>
          <button
            className="button"
            disabled={busy || !rider}
            onClick={() => void act("assign", { rider_id: rider })}
          >
            Assign rider
            <ArrowRight size={16} />
          </button>
        </>
      ) : d.status === "assigned" || d.status === "picked_up" ? (
        <button
          className="button"
          disabled={busy}
          onClick={() =>
            void act("status", {
              rider_id: d.rider_id,
              status:
                d.status === "assigned" ? "picked_up" : "out_for_delivery",
            })
          }
        >
          {d.status === "assigned" ? "Mark parcel picked up" : "Start delivery"}
          <ArrowRight size={16} />
        </button>
      ) : d.status === "out_for_delivery" ? (
        <>
          <p>
            Confirm the final handoff with a one-use code. Five-minute expiry;
            maximum five attempts.
          </p>
          <button
            className="button secondary"
            disabled={busy}
            onClick={() => void act("otp", {})}
          >
            Generate demo OTP
          </button>
          {demoOtp && (
            <div className="otp-display">
              <small>DEMO ONLY · CUSTOMER’S CODE</small>
              <strong>{demoOtp}</strong>
              <span>Visible only in the shared fictional demo.</span>
            </div>
          )}
          <form
            onSubmit={(e) => {
              e.preventDefault();
              void act("confirm", { rider_id: d.rider_id, code });
            }}
          >
            <label>
              Six-digit delivery code
              <input
                value={code}
                onChange={(e) => setCode(e.target.value)}
                inputMode="numeric"
                pattern="[0-9]{6}"
                maxLength={6}
                required
                autoComplete="one-time-code"
                placeholder="Enter OTP"
              />
            </label>
            <button className="button" disabled={busy}>
              Confirm delivery
              <CheckCircle2 size={17} />
            </button>
          </form>
        </>
      ) : (
        <div className="success">
          <CheckCircle2 />
          {d.status === "delivered"
            ? "Delivery complete. Every handoff accounted for."
            : "Delivery marked as failed."}
        </div>
      )}
      {["assigned", "picked_up", "out_for_delivery"].includes(d.status) && (
        <button
          className="text-link danger"
          disabled={busy}
          onClick={() =>
            void act("status", { rider_id: d.rider_id, status: "failed" })
          }
        >
          Mark delivery failed
        </button>
      )}
      {busy && <p role="status">Updating delivery…</p>}
    </div>
  );
}
export function RiderWorkspace() {
  const { data: riders, error, refresh: refreshRiders } = useLoad(loadRiders);
  const [selected, setSelected] = useState("");
  const [name, setName] = useState("");
  const [phone, setPhone] = useState("");
  const [registrationError, setRegistrationError] = useState("");
  const load = useCallback(
    () =>
      selected
        ? api<Delivery[]>(`/riders/${selected}/deliveries`)
        : Promise.resolve([]),
    [selected],
  );
  const { data: deliveries, error: deliveryError, refresh } = useLoad(load);
  async function register(e: React.FormEvent) {
    e.preventDefault();
    try {
      await api("/riders", { name, phone });
      setName("");
      setPhone("");
      setRegistrationError("");
      await refreshRiders();
    } catch (e) {
      setRegistrationError((e as Error).message);
    }
  }
  return (
    <main className="workspace">
      <div className="eyebrow">RIDER WORKSPACE</div>
      <h1>Ready for the next mile.</h1>
      <p>Select a demo rider to manage their assigned deliveries.</p>
      <ErrorBox message={error || deliveryError} />
      <label className="rider-select">
        Demo rider
        <select value={selected} onChange={(e) => setSelected(e.target.value)}>
          <option value="">Choose your rider</option>
          {riders?.map((r) => (
            <option key={r.id} value={r.id}>
              {r.name}
            </option>
          ))}
        </select>
      </label>
      {selected && deliveries?.length === 0 && (
        <div className="empty">No deliveries assigned to this rider yet.</div>
      )}
      <div className="rider-grid">
        {deliveries?.map((d) => (
          <section className="panel detail-card" key={d.id}>
            <div className="panel-heading">
              <Link href={`/dashboard/deliveries/${d.id}`}>
                SR-{d.id.slice(0, 8)}
              </Link>
              <Badge status={d.status} />
            </div>
            <h2>{d.recipient_name}</h2>
            <p>
              {d.postcode} · {d.formatted_address}
            </p>
            <DeliveryActions
              delivery={d}
              riders={riders || []}
              onChange={refresh}
            />
          </section>
        ))}
      </div>
      <details className="panel detail-card">
        <summary>Register another demo rider</summary>
        <ErrorBox message={registrationError} />
        <form onSubmit={register} className="form">
          <label>
            Name
            <input
              required
              value={name}
              maxLength={120}
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          <label>
            Phone
            <input
              required
              value={phone}
              type="tel"
              minLength={7}
              maxLength={30}
              onChange={(e) => setPhone(e.target.value)}
            />
          </label>
          <button className="button">Register rider</button>
        </form>
      </details>
    </main>
  );
}
