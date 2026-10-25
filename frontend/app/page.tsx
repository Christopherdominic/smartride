import Link from "next/link";
import {
  ArrowRight,
  MapPin,
  PackageCheck,
  Route,
  ShieldCheck,
  ChevronRight,
} from "lucide-react";
export default function Home() {
  return (
    <main>
      <section className="hero">
        <div>
          <div className="eyebrow">
            <span className="dot" /> THE NEXT MILE STARTS HERE
          </div>
          <h1>
            One postcode.
            <br />
            <span>Every delivery.</span>
          </h1>
          <p className="lead">
            A clearer address. A connected journey. Coordinate deliveries from
            your storefront to their doorstep with postcode-powered logistics.
          </p>
          <div className="actions">
            <Link className="button" href="/demo">
              Try the live demo <ArrowRight size={18} />
            </Link>
            <Link className="text-link" href="/dashboard">
              Explore dashboard <ChevronRight size={16} />
            </Link>
          </div>
          <div className="hero-note">
            <ShieldCheck size={17} /> Made for merchants. Connected to riders.
            Built for Nigeria.
          </div>
        </div>
        <div className="route-visual">
          <div className="visual-heading">
            <span className="eyebrow">DELIVERY, CONNECTED</span>
            <span className="pill">LIVE DEMO</span>
          </div>
          <div className="map-grid">
            <div className="map-road road-one" />
            <div className="map-road road-two" />
            <div className="map-road road-three" />
            <div className="route-line" />
            <div className="map-pin origin">
              <PackageCheck />
              <span>Your storefront</span>
            </div>
            <div className="map-pin destination">
              <MapPin />
              <span>Their doorstep</span>
            </div>
            <div className="postcode-card">
              <small>THE ADDRESS THAT CONNECTS IT ALL</small>
              <strong>EK01 A03 FK01</strong>
              <span>
                <span className="dot" /> Illustrative postcode
              </span>
            </div>
          </div>
          <div className="visual-bottom">
            <span>
              <span className="dot" /> One connected delivery journey
            </span>
            <Route size={20} />
          </div>
        </div>
      </section>
      <section className="trust-strip">
        <span>BUILT AROUND YOUR LAST MILE</span>
        <strong>Merchants</strong>
        <strong>Logistics teams</strong>
        <strong>Delivery riders</strong>
        <strong>Happy customers</strong>
      </section>
      <section className="section">
        <div className="eyebrow">LESS FRICTION. MORE DELIVERIES.</div>
        <h2>
          From “where exactly?”
          <br />
          to right at the door.
        </h2>
        <div className="feature-grid">
          {[
            [
              MapPin,
              "Start with a postcode",
              "Verify postcode validity and collect customer-provided delivery details before dispatch.",
            ],
            [
              Route,
              "Keep everyone connected",
              "Create orders, assign riders, and follow every handoff from a single workspace.",
            ],
            [
              ShieldCheck,
              "Close the loop, securely",
              "Confirm the final handoff with a time-limited, one-use delivery OTP.",
            ],
          ].map(([Icon, title, description], i) => {
            const I = Icon as typeof MapPin;
            return (
              <article className="feature" key={i}>
                <I />
                <h3>{String(title)}</h3>
                <p>{String(description)}</p>
              </article>
            );
          })}
        </div>
      </section>
      <section className="concept">
        <div>
          <div className="eyebrow">A SIMPLE IDEA. A BETTER LAST MILE.</div>
          <h2>
            A postcode is just
            <br />
            the beginning.
          </h2>
          <p>
            SmartRide demonstrates how Nigeria’s NIPOST NDAPS concept could
            connect address identification to delivery operations. This
            integration verifies postcode validity at L1. Customers supply their
            street address and directions separately; these details are not
            verified by NIPOST. Offline fixtures remain available in mock mode.
          </p>
        </div>
        <ol className="steps">
          {[
            "Verify a digital postcode",
            "Create and assign a delivery",
            "Track progress through each handoff",
            "Confirm arrival with an OTP",
          ].map((s, i) => (
            <li key={s}>
              <span>0{i + 1}</span>
              {s}
              <ArrowRight size={18} />
            </li>
          ))}
        </ol>
      </section>
      <section className="bottom-cta">
        <h2>Your next delivery starts here.</h2>
        <p>
          Take a parcel from postcode to proof of delivery in three minutes.
        </p>
        <Link href="/demo" className="button">
          Start the guided demo <ArrowRight size={18} />
        </Link>
      </section>
    </main>
  );
}
